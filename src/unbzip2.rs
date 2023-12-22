use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use neon::prelude::*;
use neon::types::buffer::TypedArray;
use bzip2::write::BzDecoder;

#[derive(Clone)]
pub struct Unbzip2Stream {
    decoder: Arc<Mutex<BzDecoder<Vec<u8>>>>
}
impl Unbzip2Stream {
    pub fn new() -> Self {
        Self {
            decoder: Arc::new(Mutex::new(BzDecoder::new(Vec::new())))
        }
    }
    // ストリームへの可変参照を取得しようとします
    fn lock(&self) -> Result<MutexGuard<BzDecoder<Vec<u8>>>, Unbzip2Error> {
        // `lock`の代わりに`try_lock`を使用します。 複数の同時呼び出しはエンコーダーに対して未定義です。
        // 呼び出し元は、ストリームに直列に書き込むように注意する必要があります。
        // `Transform`は、バックプレッシャーとバッファリングを適用することで、この保証を提供します。
        // `Mutex`は常にロック解除されている必要があります。
        Ok(self.decoder.try_lock()?)
    }
    pub fn write(self, data: Vec<u8>) -> Result<Self, Unbzip2Error> {
        self.lock()?.write_all(&data)?;
        Ok(self)
    }
    // Finish compressing. Multiple calls to this function will error or panic.
    fn finish(self) -> Result<Self, Unbzip2Error> {
        self.lock()?.try_finish()?;
        Ok(self)
    }
    pub fn output(self) -> Result<Vec<u8>, Unbzip2Error> {
        let mut guard = self.lock()?;
        let data = guard.get_mut();
        let output = data.clone();
        data.truncate(0);
        Ok(output)
    }
    // これは、`compress_chunk`と`compress_finish`の両方で`TaskBuilder :: promise`のコールバックとして使用される小さなヘルパーです。
    // `CompressStream :: output`を使用して書き込まれたバイトを取得し、データを`ArrayBuffer`に入れて、Rustエラーが発生した場合はJavaScript例外をスローします。
    fn and_buffer(
        mut cx: TaskContext,
        // `cx.task(..)`クロージャの戻り値
        result: Result<Self, Unbzip2Error>,
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
impl Finalize for Unbzip2Stream {}

#[derive(Debug)]
// すべてのエラーは、`Display`実装を`Error`メッセージとして使用してJavaScript例外に変換されます。
pub struct Unbzip2Error(String);
impl<T> From<TryLockError<T>> for Unbzip2Error {
    fn from(err: TryLockError<T>) -> Self {
        Self(err.to_string())
    }
}
impl From<std::io::Error> for Unbzip2Error {
    fn from(err: std::io::Error) -> Self {
        Self(err.to_string())
    }
}
impl std::fmt::Display for Unbzip2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Unbzip2Error {}



// JavaScriptに渡すことができるボックス化された`Unbzip2Stream`を作成します
pub fn unbzip2_create(mut cx: FunctionContext) -> JsResult<JsBox<Unbzip2Stream>> {
    let stream = Unbzip2Stream::new();
    // `cx.boxed`はFFI境界を越えることができる不透明なポインタを作成します
    Ok(cx.boxed(stream))
}

// Nodeワーカースレッドプールでデータのチャンクを圧縮し、圧縮されたデータを含む可能性のあるプロミスを返します。
pub fn unbzip2_chunk(mut cx: FunctionContext) -> JsResult<JsPromise> {
    // これは少し変わった構文ですが、関数の最初の引数を取り、それを`JsBox <Unbzip2Stream>`としてダウンキャストし、
    // 最後に`Unbzip2Stream`の`Clone`実装を呼び出します。 `& **`はいくつかのスマートポインタによるものです。
    // スマートポインタは、[`Deref`]（https://doc.rust-lang.org/std/ops/trait.Deref.html）トレイトを実装する型です。
    //
    // 外側の型は`neon :: handle :: Handle`です。 `Handle`は、NeonがJavaScript値への参照をガベージコレクションされた後に保持できないようにするために使用する型です。
    // 次の型は`JsBox`で、JavaScriptのRustデータへの参照を保持するためのスマートポインタです。 `**`はこれらの2つの型をデリファレンスし、`Unbzip2Stream`を与えます。
    // ただし、`JsBox`から移動することはできないため、`&`で参照をすぐに取得します。 最後に、`Unbzip2Stream`の`clone`実装を呼び出すことができます。
    let stream = (&**cx.argument::<JsBox<Unbzip2Stream>>(0)?).clone();

    // 2番目の引数は`encoding`です。 ただし、gunzip2はエンコーディングに依存しないため、必要ありません。
    // let encoding = cx.argument::<JsString>(1)?;

    // 3番目の引数を`Uint8Array`として取得します。 データは、`&[u8]`として借用し、クローンすることですぐに`Vec<u8>`に変換されます。
    let chunk = cx.argument::<JsTypedArray<u8>>(2)?.as_slice(&cx).to_vec();

    let promise = cx
        // Nodeワーカースレッドプールで実行するタスクを作成します
        .task(move || stream.write(chunk))
        // タスクの結果を`ArrayBuffer`に変換し、JavaScriptのメインスレッドでプロミスを解決します。
        .promise(Unbzip2Stream::and_buffer);

    Ok(promise)
}

// Complete compressing the data and get the remaining output
pub fn unbzip2_finish(mut cx: FunctionContext) -> JsResult<JsPromise> {
    // Get a shallow clone of `Unbzip2Stream`; same as in `unzip2_chunk`
    // This is an alternative to the `&**` syntax used earlier. Instead, it uses auto-deref
    // and universal call syntax for the `clone` call to coerce to proper type.
    let stream = Unbzip2Stream::clone(&&cx.argument::<JsBox<Unbzip2Stream>>(0)?);

    let promise = cx
        // Finish the stream on the Node worker pool
        .task(move || stream.finish())
        // Convert the remaining output into an `ArrayBuffer` and resolve the promise
        // on the JavaScript main thread.
        .promise(Unbzip2Stream::and_buffer);

    Ok(promise)
}

