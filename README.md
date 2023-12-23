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
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
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
bzip2 (Rust): 7.208s
unbzip2-stream (JS): 16.139s
bzip2 command: 3.577s
lbzip2 command: 573.724ms
pbzip2 command: 3.558s
```
