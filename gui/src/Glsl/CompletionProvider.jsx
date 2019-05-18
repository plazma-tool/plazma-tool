import { GlslReservedVariables, GlslPredefinedConstants, GlslBuiltinFunctions, GlslLanguageSnippets,
    ShadertoyVariables } from './CompletionData';

function glsl_suggestions(model, position) {

    let suggestions = [
        GlslLanguageSnippets,
        GlslReservedVariables,
        GlslPredefinedConstants,
        GlslBuiltinFunctions,
        ShadertoyVariables,
    ].flat();

    return { suggestions: suggestions };
};

export const GlslCompletionProvider = {
    provideCompletionItems: glsl_suggestions,
};


