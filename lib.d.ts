// 型定義ファイルは自分で書きましょう

declare class PassThroughStream {
  #private;
}

export declare const createPassThroughStream: () => PassThroughStream;

export declare const handlePassThroughStreamChunk: (
  stream: PassThroughStream,
  encoding: string,
  chunk: Uint8Array,
) => Promise<Uint8Array>;

declare class Unbzip2 {
  #private;
}

export declare const ungzip2Create: () => Unbzip2;

export declare const ungzip2Chunk: (
  unbzip2: Unbzip2,
  encoding: string,
  chunk: Uint8Array,
) => Promise<Uint8Array>;

export declare const ungzip2Finish: (unbzip2: Unbzip2) => Promise<Uint8Array>;
