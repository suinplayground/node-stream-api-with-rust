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
  console.time("unzipWithRust");
  await unzipWithRust();
  console.timeEnd("unzipWithRust");

  console.time("unzipWithJavaScript");
  await unzipWithJavaScript();
  console.timeEnd("unzipWithJavaScript");
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
