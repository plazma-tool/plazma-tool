// FIXME load yaml directly
//import doc from './data/GlslReservedVariables.yaml';

import language_snippets_doc from './data/GlslLanguageSnippets.json';
import reserved_variables_doc from './data/GlslReservedVariables.json';
import predefined_constants_doc from './data/GlslPredefinedConstants.json';
import sampler_variables_doc from './data/GlslSamplerVariables.json';
import image_variables_doc from './data/GlslImageVariables.json';
import builtin_functions_doc from './data/GlslBuiltinFunctions.json';
import vector_constructors_doc from './data/GlslVectorConstructors.json';
import matrix_constructors_doc from './data/GlslMatrixConstructors.json';
import shadertoy_variables_doc from './data/ShadertoyVariables.json';

// process the json data
function p(doc) {
    return doc.map((i) => {
        return {
            label: i.label,
            detail: (i.detail ? i.detail : null),
            insertText: (i.insertText ? i.insertText : i.label),
            documentation: (i.documentation ? i.documentation : null),
            kind: (i.kind ? i.kind : 17), // 17 = Keyword, 18 = Text, 25 = Snippet
            insertTextRules: (i.insertTextRules ? i.insertTextRules : null),
        };
    });
};

export const GlslLanguageSnippets    = p(language_snippets_doc);
export const GlslReservedVariables   = p(reserved_variables_doc);
export const GlslPredefinedConstants = p(predefined_constants_doc);
export const GlslSamplerVariables    = p(sampler_variables_doc);
export const GlslImageVariables      = p(image_variables_doc);
export const GlslBuiltinFunctions    = p(builtin_functions_doc);
export const GlslVectorConstructors  = p(vector_constructors_doc);
export const GlslMatrixConstructors  = p(matrix_constructors_doc);
export const ShadertoyVariables      = p(shadertoy_variables_doc);

