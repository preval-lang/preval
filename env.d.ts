type IO = {
    println: (message: string) => void;
};

declare const compile_io: IO;
declare const compiler: {
    executable: (name: string, callback: (io: IO) => void) => void;
    library: (name: string, object: { [name: string]: (...args: any[]) => void }) => void;
};

declare const println: (message: string) => void;