use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use neon::prelude::*;
use neon::types::buffer::TypedArray;

#[derive(Clone)]
pub struct PassThroughStream {
    vec: Arc<Mutex<Vec<u8>>>
}
impl PassThroughStream {
    pub fn new() -> Self {
        let vec = Vec::new();
        Self {
            vec: Arc::new(Mutex::new(vec))
        }
    }
    // ストリームへの可変参照を取得しようとします
    fn lock(&self) -> Result<MutexGuard<Vec<u8>>, PassThroughError> {
        // `lock`の代わりに`try_lock`を使用します。 複数の同時呼び出しはエンコーダーに対して未定義です。
        // 呼び出し元は、ストリームに直列に書き込むように注意する必要があります。
        // `Transform`は、バックプレッシャーとバッファリングを適用することで、この保証を提供します。
        // `Mutex`は常にロック解除されている必要があります。
        Ok(self.vec.try_lock()?)
    }
    // データのチャンクをvecに書き込みます
    pub fn write(self, data: Vec<u8>) -> Result<Self, PassThroughError> {
        println!("[rust] write: {:?}", String::from_utf8(data.clone()).unwrap());
        self.lock()?.extend(&data[0..]);
        Ok(self)
    }
    // vecに書き込まれたデータを取得し、vecをクリアします
    pub fn output(self) -> Result<Vec<u8>, PassThroughError> {
        let mut guard  = self.lock()?;
        let mut output = vec![];
        for item in guard.iter() {
            output.push(*item);
        }
        guard.clear();
        println!("[rust] read: {:?}", String::from_utf8(output.clone()).unwrap());
        Ok(output)
    }
    // これは、`compress_chunk`と`compress_finish`の両方で`TaskBuilder :: promise`のコールバックとして使用される小さなヘルパーです。
    // `CompressStream :: output`を使用して書き込まれたバイトを取得し、データを`ArrayBuffer`に入れて、Rustエラーが発生した場合はJavaScript例外をスローします。
    fn and_buffer(
        mut cx: TaskContext,
        // `cx.task(..)`クロージャの戻り値
        result: Result<Self, PassThroughError>,
    ) -> JsResult<JsBuffer> {
        let output = result
            // エラーが発生する可能性があります。 書き込まれたデータを条件付きで取得します
            .and_then(|stream| stream.output())
            // RustのエラーをJavaScriptの例外に変換する
            .or_else(|err| cx.throw_error(err.to_string()))?;

        // 書き込まれたデータを含む`Vec<u8>`でバックアップされた`Buffer`を作成します
        Ok(JsBuffer::external(&mut cx, output))
    }
}
// `JsBox`に配置された型は、RustからJavaScriptにRustデータを渡すための不透明なポインタである`Finalize`トレイトを実装する必要があります。
//
// `Finalize`トレイトは、値がガベージコレクションされるときにコードを実行するためのフックをオプションで提供します。
impl Finalize for PassThroughStream {}

#[derive(Debug)]
// すべてのエラーは、`Display`実装を`Error`メッセージとして使用してJavaScript例外に変換されます。
pub struct PassThroughError(String);
impl<T> From<TryLockError<T>> for PassThroughError {
    fn from(err: TryLockError<T>) -> Self {
        Self(err.to_string())
    }
}
impl std::fmt::Display for PassThroughError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for PassThroughError {}



// JavaScriptに渡すことができるボックス化された`PassThroughStream`を作成します
pub fn create_pass_through_stream(mut cx: FunctionContext) -> JsResult<JsBox<PassThroughStream>> {
    let stream = PassThroughStream::new();
    // `cx.boxed`はFFI境界を越えることができる不透明なポインタを作成します
    Ok(cx.boxed(stream))
}

// Nodeワーカースレッドプールでデータのチャンクを圧縮し、圧縮されたデータを含む可能性のあるプロミスを返します。
pub fn handle_pass_through_stream_chunk(mut cx: FunctionContext) -> JsResult<JsPromise> {
    // これは少し変わった構文ですが、関数の最初の引数を取り、それを`JsBox <PassThroughStream>`としてダウンキャストし、
    // 最後に`PassThroughStream`の`Clone`実装を呼び出します。 `& **`はいくつかのスマートポインタによるものです。
    // スマートポインタは、[`Deref`]（https://doc.rust-lang.org/std/ops/trait.Deref.html）トレイトを実装する型です。
    //
    // 外側の型は`neon :: handle :: Handle`です。 `Handle`は、NeonがJavaScript値への参照をガベージコレクションされた後に保持できないようにするために使用する型です。
    // 次の型は`JsBox`で、JavaScriptのRustデータへの参照を保持するためのスマートポインタです。 `**`はこれらの2つの型をデリファレンスし、`PassThroughStream`を与えます。
    // ただし、`JsBox`から移動することはできないため、`&`で参照をすぐに取得します。 最後に、`PassThroughStream`の`clone`実装を呼び出すことができます。
    let stream = (&**cx.argument::<JsBox<PassThroughStream>>(0)?).clone();

    // 2番目の引数は`encoding`です。 ただし、gzipはエンコーディングに依存しないため、必要ありません。
    // let encoding = cx.argument::<JsString>(1)?;

    // 3番目の引数を`Uint8Array`として取得します。 データは、`&[u8]`として借用し、クローンすることですぐに`Vec<u8>`に変換されます。
    let chunk = cx.argument::<JsTypedArray<u8>>(2)?.as_slice(&cx).to_vec();

    let promise = cx
        // Nodeワーカースレッドプールで実行するタスクを作成します
        .task(move || stream.write(chunk))
        // タスクの結果を`ArrayBuffer`に変換し、JavaScriptのメインスレッドでプロミスを解決します。
        .promise(PassThroughStream::and_buffer);

    Ok(promise)
}

