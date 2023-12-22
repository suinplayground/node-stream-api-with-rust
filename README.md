# @suinplayground/node-stream-api

Node.js Stream API互換のライブラリをRustで書くデモです。

## デモの実行方法

```shell
yarn install
yarn run dev
```

### デモの実行結果

```
❯ yarn dev
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
Streamを受け流すだけのデモ
[rust] write: "hello"
[rust] read: "hello"
{ chunk: 'hello' }
[rust] write: "world"
[rust] read: "world"
{ chunk: 'world' }
Bzip2を解凍するデモ
{ chunk: 'hello world\nline second\nline third\n' }
Bzip2を解凍するベンチマーク: RustとJavaScriptどっちが速いか？
unzipWithRust: 8.907s
unzipWithJavaScript: 20.803s
```