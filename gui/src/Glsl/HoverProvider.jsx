
export const GlslHoverProvider = {
    provideHover: function (model, position, token) {
        // TODO make an xhr call to the server with the position, it should return one or
        // more markdown snippets
        if (position.lineNumber === 1) {
            return {
                contents: [
                    { value: 'She said *WHAT*?'}
                ]
            }
        } else if (position.lineNumber === 2) {
            return {
                contents: [
                    { value: 'He said **WHO**?'}
                ]
            }
        }
        return null;
    }
};
