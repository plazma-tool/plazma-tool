// FIXME load yaml directly
//import doc from './data/GlslReservedVariables.yaml';

import reserved_variables_doc from './data/GlslReservedVariables.json';
import sampler_variables_doc from './data/GlslSamplerVariables.json';
import image_variables_doc from './data/GlslImageVariables.json';
import builtin_functions_doc from './data/GlslBuiltinFunctions.json';
import vector_constructors_doc from './data/GlslVectorConstructors.json';
import matrix_constructors_doc from './data/GlslMatrixConstructors.json';
import shadertoy_variables_doc from './data/ShadertoyVariables.json';

export const GlslReservedVariables = reserved_variables_doc;
export const GlslSamplerVariables = sampler_variables_doc;
export const GlslImageVariables = image_variables_doc;
export const GlslBuiltinFunctions = builtin_functions_doc;
export const GlslVectorConstructors = vector_constructors_doc;
export const GlslMatrixConstructors = matrix_constructors_doc;
export const ShadertoyVariables = shadertoy_variables_doc;


