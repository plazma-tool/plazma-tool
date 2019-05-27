// @flow

import { GlslSamplerVariables, GlslPredefinedConstants, GlslImageVariables, GlslBuiltinFunctions,
    ShadertoyVariables, GlslVectorConstructors, GlslMatrixConstructors } from './CompletionData';

export const GlslTokensProvider = {

    // red color to show what is not being tokenized
    //defaultToken: 'glsl-illegal',

    defaultToken: '',

    symbols:  /[=><!~?:&|+\-*/^%]+/,

    brackets: [
        ['{','}','delimiter.curly'],
        ['[',']','delimiter.square'],
        ['(',')','delimiter.parenthesis'],
    ],

    shadertoy_variables: ShadertoyVariables.map((i) => i.label),

    glsl_keywords: [
        'break',
        'case',
        'continue',
        'default',
        'discard',
        'do',
        'else',
        'false',
        'for',
        'if',
        'return',
        'switch',
        'true',
        'while',
    ],

    glsl_macros: [
        '#define',
        '#defined',
        '#elif',
        '#else',
        '#endif',
        '#error',
        '#extension',
        '#if',
        '#ifdef',
        '#ifndef',
        '#include',
        '#line',
        '#pragma',
        '#undef',
        '#version',
    ],

    glsl_precision_modifiers: [
        'highp',
        'lowp',
        'mediump',
        'precision',
    ],

    glsl_predefined_constants: GlslPredefinedConstants.map((i) => i.label),

    glsl_operators: [
        '!',
        '!=',
        '%',
        '&&',
        '&',
        '*',
        '*=',
        '+',
        '++',
        '+=',
        '-',
        '--',
        '/',
        '<',
        '<<',
        '<=',
        '=',
        '==',
        '>',
        '>=',
        '>>',
        '^',
        '^^',
        '|',
        '||',
        '~',
    ],

    glsl_simple_types: [
        'atomic_uint',
        'bool',
        'double',
        'float',
        'int',
        'struct',
        'uint',
        'void',
    ],

    glsl_sampler_variables: GlslSamplerVariables.map((i) => i.label),
    glsl_image_variables: GlslImageVariables.map((i) => i.label),

    glsl_storage_modifiers: [
        'attribute',
        'binding',
        'buffer',
        'centroid',
        'coherent',
        'const',
        'flat',
        'in',
        'inout',
        'invariant',
        'layout',
        'location',
        'noperspective',
        'out',
        'patch',
        'readonly',
        'sampler',
        'shared',
        'smooth',
        'uniform',
        'varying',
        'writeonly',
    ],

    glsl_vector_constructors: GlslVectorConstructors.map((i) => i.label),
    glsl_matrix_constructors: GlslMatrixConstructors.map((i) => i.label),
    glsl_builtin_functions: GlslBuiltinFunctions.map((i) => i.label),

    glsl_illegals: [
        'asm', 'enum', 'extern', 'goto', 'inline', 'long', 'short', 'sizeof', 'static',
        'typedef', 'union', 'unsigned',
    ],

    tokenizer: {

        root: [
            // OpenGL builtin uniforms and constants
            [/gl_[A-Z][\w]+/, 'glsl-reserved-variable' ],

            // constants such as __VERSION__ or GL_es_profile
            [/__[A-Z]+__/, { cases: { '@glsl_predefined_constants': 'glsl-predefined-constant' } }],
            [/GL_[A-Za-z][\w_$]+/, { cases: { '@glsl_predefined_constants': 'glsl-predefined-constant' } }],

            // Shadertoy builtin uniforms
            [/iChannel\d/, 'shadertoy-variable'],
            [/i[A-Z][\w]+/, { cases: { '@shadertoy_variables': 'shadertoy-variable' } }],

            // macros
            [/#[a-z$][\w$]*/, { cases: { '@glsl_macros': 'glsl-macro' } }],

            // 'g_' or 'i_' prefix can be used for global variables
            [/[giGI]_[\w_$][\w$]*/, 'global-variable' ],
            // uppercase globals (PI, MAX_DIST), or struct types (Ray)
            [/[A-Z][\w$]*/, 'upcase-name' ],

            // UI widget syntax such as "// ui_color"
            [/\/\/ +ui_\w+ *$/, 'comment-ui-widget'],

            // user function names
            //
            // The nested matching within the function's parens () is tough because of the
            // state transitions. The Monarch example for 'koka' has something similar for
            // type definitions.
            //
            // Not highlighting it, it adds too much color everywhere anyway. Let it be the
            // same as text.

            // identifiers and keywords
            [/[a-z_$][\w$]*/, { cases: {
                '@glsl_keywords': 'glsl-keyword',
                '@glsl_precision_modifiers': 'glsl-precision-modifier',
                '@glsl_simple_types': 'glsl-simple-type',
                '@glsl_vector_constructors': 'glsl-vector-constructor',
                '@glsl_matrix_constructors': 'glsl-matrix-constructor',
                '@glsl_sampler_variables': 'glsl-sampler-variable',
                '@glsl_image_variables': 'glsl-image-variable',
                '@glsl_storage_modifiers': 'glsl-storage-modifier',
                '@glsl_builtin_functions': 'glsl-builtin-function',
                '@glsl_illegals': 'glsl-illegal',
                '@default': 'identifier',
            } }],

            // trailing whitespace
            [/[ \t\r\n]+$/, 'trailing-whitespace'],

            // whitespace
            { include: '@whitespace' },

            // delimiters and operators
            [/[{}()[\]]/, '@brackets'],

            // swizzle operators
            [/\.[xyzwrgba]+\b/, 'glsl-swizzle'],

            [/@symbols/, { cases: {
                '@glsl_operators': 'glsl-operator',
                '@default' : '',
            } }],

            // numbers
            [/\d*\.\d+([eE][-+]?\d+)?/, 'number.float'],
            [/0[xX][0-9a-fA-F]+/, 'number.hex'],
            [/\d+/, 'number'],

            // delimiter: after number because of .\d floats
            [/[;,.]/, 'delimiter'],
        ],

        comment: [
            [/[^/*]+/, 'comment' ],
            [/\/\*/,    'comment', '@push' ], // nested comment
            ["\\*/",    'comment', '@pop'  ],
            [/[/*]/,   'comment' ]
        ],

        // there are no strings in glsl
        string: [],

        whitespace: [
            [/[ \t\r\n]+/, 'white'],
            [/\/\*/,       'comment', '@comment' ],
            [/\/\/.*$/,    'comment'],
        ],
    },

};

