declare module 'encoding-japanese' {
    export type Encoding =
        'UTF32'   | 'UTF16'  | 'UTF16BE' |
        'UTF16LE' | 'BINARY' | 'ASCII'   |
        'JIS'     | 'UTF8'   | 'EUCJP'   |
        'SJIS'    | 'UNICODE';
    type RawType = Uint8Array | number[] | Buffer;

    interface ConvertOptions {
        to: Encoding;
        from?: Encoding;
        type?: 'string' | 'arraybuffer' | 'array';
        bom?: boolean;
    }

    export function detect(data: RawType, encodings?: Encoding | Encoding[]): Encoding;
    export function convert(data: RawType, to: Encoding, from?: Encoding): number[];
    export function convert(data: RawType, options: ConvertOptions): string | ArrayBuffer | number[];
    export function urlEncode(data: number[] | Uint8Array): string;
    export function urlDecode(data: string): number[];
    export function base64Encode(data: number[] | Uint8Array): string;
    export function base64Decode(data: string): number[];
    export function codeToString(data: number[] | Uint8Array): string;
    export function stringToCode(data: string): number[];
    export function toHankakuCase(data: number[] | string): number[] | string;
    export function toZenkakuCase(data: number[] | string): number[] | string;
    export function toHiraganaCase(data: number[] | string): number[] | string;
    export function toKatakanaCase(data: number[] | string): number[] | string;
    export function toHankanaCase(data: number[] | string): number[] | string;
    export function toZenkanaCase(data: number[] | string): number[] | string;
    export function toHankakuSpace(data: number[] | string): number[] | string;
    export function toZenkakuSpace(data: number[] | string): number[] | string;
}
