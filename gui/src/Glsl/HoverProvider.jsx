import { GlslReservedVariables, GlslPredefinedConstants, GlslBuiltinFunctions, GlslLanguageSnippets,
    ShadertoyVariables } from './CompletionData';

export const GlslHoverProvider = {
    provideHover: function (model, position) {
        let results = [];

        let word = model.getWordAtPosition(position);
        if (word !== null) {
            results = [

                GlslLanguageSnippets,
                GlslReservedVariables,
                GlslPredefinedConstants,
                GlslBuiltinFunctions,
                ShadertoyVariables,

            ].flat().filter((i) => {
                // filter the flattened list for matches

                if (i.label === word.word && (i.detail !== null || i.documentation !== null)) {
                    return true;
                } else {
                    return false;
                }

            }).map((i) => {
                // map the results to the documentation (markdown string { value: '...' }) or the
                // detail (plain string)

                if (i.documentation !== null) {
                    return i.documentation;
                }
                return { value: i.detail };

            });
        }

        return { contents: results };
    }
};
