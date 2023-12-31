import {
  createPassThroughStream,
  handlePassThroughStreamChunk,
  ungzip2Create,
  ungzip2Chunk,
  ungzip2Finish,
} from "./lib";
import { Readable, Writable, Transform } from "node:stream";
import { pipeline } from "node:stream/promises";
import { createReadStream, createWriteStream } from "node:fs";
import unbzip2Stream from "unbzip2-stream";
import { ChildProcess, spawn } from "node:child_process";

main().catch((err) => console.error(err));

async function main() {
  // Streamを受け流すだけのデモ
  await runPassThroughDemo();
  // Bzip2を解凍するデモ
  await runUnbzip2Demo();
  // Bzip2を解凍するベンチマーク: RustとJavaScriptどっちが速いか？
  await runUnbzip2Benchmark();
}

async function runPassThroughDemo() {
  console.info("Streamを受け流すだけのデモ");
  const input = new Readable({
    read() {
      this.push("hello");
      this.push("world");
      this.push(null);
    },
  });

  const output = new Writable({
    write(chunk, encoding, callback) {
      console.log({ chunk: chunk.toString() });
      callback();
    },
  });

  const passThroughStream = createPassThroughStream();
  const transform = new Transform({
    transform(chunk, encoding, callback) {
      handlePassThroughStreamChunk(passThroughStream, encoding, chunk)
        .then((chunk) => callback(null, chunk))
        .catch((err) => callback(err));
    },
  });

  await pipeline(input, transform, output);
}

async function runUnbzip2Demo() {
  console.info("Bzip2を解凍するデモ");
  const file = createReadStream("hello-world.bz2");
  const output = new Writable({
    write(chunk, encoding, callback) {
      console.log({ chunk: chunk.toString() });
      callback();
    },
  });
  const unbzip2 = ungzip2Create();
  const transform2 = new Transform({
    transform(chunk, encoding, callback) {
      ungzip2Chunk(unbzip2, encoding, chunk)
        .then((chunk) => callback(null, chunk))
        .catch((err) => callback(err));
    },
    flush(callback) {
      ungzip2Finish(unbzip2)
        .then((chunk) => callback(null, chunk))
        .catch((err) => callback(err));
    },
  });

  await pipeline(file, transform2, output);
}

async function runUnbzip2Benchmark() {
  console.info("Bzip2を解凍するベンチマーク: RustとJavaScriptどっちが速いか？");
  console.time("bzip2 (Rust)");
  await unzipWithRust();
  console.timeEnd("bzip2 (Rust)");

  console.time("unbzip2-stream (JS)");
  await unzipWithJavaScript();
  console.timeEnd("unbzip2-stream (JS)");

  console.time("bzip2 command");
  await unzipWithBzip2();
  console.timeEnd("bzip2 command");

  console.time("lbzip2 command");
  await unzipWithLbzip2();
  console.timeEnd("lbzip2 command");

  console.time("pbzip2 command");
  await unzipWithPbzip2();
  console.timeEnd("pbzip2 command");
}

async function unzipWithRust() {
  const file = createReadStream("large.bz2");
  const output = createWriteStream("extracted-data-with-rust.data");
  const unbzip2 = ungzip2Create();
  const transform2 = new Transform({
    transform(chunk, encoding, callback) {
      ungzip2Chunk(unbzip2, encoding, chunk)
        .then((chunk) => callback(null, chunk))
        .catch((err) => callback(err));
    },
    flush(callback) {
      ungzip2Finish(unbzip2)
        .then((chunk) => callback(null, chunk))
        .catch((err) => callback(err));
    },
  });

  await pipeline(file, transform2, output);
}

async function unzipWithJavaScript() {
  const file = createReadStream("large.bz2");
  const output = createWriteStream("extracted-data-with-unbzip2-stream.data");
  const unbzip2 = unbzip2Stream();
  await pipeline(file, unbzip2, output);
}

async function unzipWithBzip2() {
  const file = createReadStream("large.bz2");
  const output = createWriteStream("extracted-data-with-bzip2.data");
  const command = spawn("bzip2", ["-d", "-c"], {
    stdio: ["pipe", "pipe", "inherit"],
  });
  await pipeline(file, createTransform(command), output);
}

async function unzipWithLbzip2() {
  const file = createReadStream("large.bz2");
  const output = createWriteStream("extracted-data-with-lbzip2.data");
  const command = spawn("lbzip2", ["-d", "-c"], {
    stdio: ["pipe", "pipe", "inherit"],
  });
  await pipeline(file, createTransform(command), output);
}

async function unzipWithPbzip2() {
  const file = createReadStream("large.bz2");
  const output = createWriteStream("extracted-data-with-pbzip2.data");
  const command = spawn("pbzip2", ["-d", "-c"], {
    stdio: ["pipe", "pipe", "inherit"],
  });
  await pipeline(file, createTransform(command), output);
}

// from: https://blog.lufia.org/entry/2021/09/26/113000
function createTransform(p: ChildProcess): Transform {
  const data: string[] = [];
  p.stdout?.on("data", (s) => {
    data.push(s);
  });

  const t = new Transform({
    transform: (s, encoding, callback): void => {
      p.stdin?.write(s);
      while (data.length > 0) t.push(data.shift());
      callback();
    },
    final: async (callback): Promise<void> => {
      p.stdin?.end();
      const status = await new Promise((resolve, reject) => {
        p.on("close", resolve);
      });
      if (status !== 0) throw new Error(`${p.spawnfile}: exit with ${status}`);
      while (data.length > 0) t.push(data.shift());
      callback();
    },
  });
  return t;
}
