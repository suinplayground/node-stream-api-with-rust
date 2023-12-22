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
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
[rust] write: "hello"
[rust] read: "hello"
hello
[rust] write: "world"
[rust] read: "world"
world
```