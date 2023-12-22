import { createPassThroughStream, handlePassThroughStreamChunk } from "./lib";
import { pipeline, Readable, Writable, Transform } from "node:stream";

const input = new Readable({
  read() {
    this.push("hello");
    this.push("world");
    this.push(null);
  },
});

const output = new Writable({
  write(chunk, encoding, callback) {
    console.log(chunk.toString());
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

pipeline(input, transform, output, (err) => {
  if (err) {
    console.error(err);
  }
});
