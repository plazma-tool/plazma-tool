import { GlslReservedVariables, GlslPredefinedConstants, GlslBuiltinFunctions, GlslLanguageSnippets,
    ShadertoyVariables } from './CompletionData';

function glsl_suggestions(model, position) {
    let results = [];

    let i_word = model.getWordAtPosition(position);
    if (i_word !== null && i_word.word.length > 2) {
        let word = i_word.word;
        if (word.startsWith('gl_')) {
            results = GlslReservedVariables;
        } else {
            results = [
                GlslLanguageSnippets,
                GlslPredefinedConstants,
                GlslBuiltinFunctions,
                ShadertoyVariables,
            ].flat();
        }
    }

    return { suggestions: results };
};

export const GlslCompletionProvider = {
    provideCompletionItems: glsl_suggestions,
};


