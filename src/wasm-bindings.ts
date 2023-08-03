
export { }

interface IBindingsJs {
    log(message: string): void;
}

declare global {
    var BindingsJs: IBindingsJs;
}

globalThis.BindingsJs = {
    log: (message: string): void => {
        console.log(message);
    }
}

