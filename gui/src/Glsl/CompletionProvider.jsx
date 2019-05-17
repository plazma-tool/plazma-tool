import { GlslReservedVariables } from './CompletionData';

function get_glsl_suggestions(monaco) {
    return function glsl_suggestions(model, position, token) {
        let suggestions = [];

        let a = GlslReservedVariables.map((i) => {
            return {
                label: i.label,
                detail: i.label,// TODO
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: i.label,
                documentation: { value: 'GLSL Reserved Variable or Constant' },// TODO
            };
        });

        suggestions = suggestions.concat(a);

        return { suggestions: suggestions };
    };
}

export function GetGlslCompletionProvider(monaco) {
    return {
        provideCompletionItems: get_glsl_suggestions(monaco),
    };
};


