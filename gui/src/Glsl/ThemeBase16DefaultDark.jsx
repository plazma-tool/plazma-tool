// @flow

// Base16 Deafult Dark theme
const c = {
    'base00': "181818",
    'base01': "282828",
    'base02': "383838",
    'base03': "585858",
    'base04': "b8b8b8",
    'base05': "d8d8d8",
    'base06': "e8e8e8",
    'base07': "f8f8f8",
    'base08': "ab4642",
    'base09': "dc9656",
    'base0A': "f7ca88",
    'base0B': "a1b56c",
    'base0C': "86c1b9",
    'base0D': "7cafc2",
    'base0E': "ba8baf",
    'base0F': "a16946",
};

export const ThemeBase16DefaultDark = {
    base: 'vs-dark',
    inherit: true,
    rules: [
        { token: '', foreground: c['base05'] },// default text color
        { token: 'identifier', foreground: c['base05'] },
        { token: 'white', background: c['base00'] },
        { token: 'symbol', foreground: c['base0E'] },

        //{ token: 'user-function-name', foreground: c['base0B'] },

        { token: 'trailing-whitespace', foreground: 'ff0000', fontStyle: 'underline' },

        // dots, etc
        { token: 'delimiter', foreground: c['base05'] },
        // braces, brackets, parens
        // add some color to break up the white text and parens in sth like
        // res = opSmin(res, sdCylinder((p - vec3(.0, .08, .0)), .008, .02), 0.02);
        { token: 'delimiter.curly', foreground: c['base0C'] },
        { token: 'delimiter.square', foreground: c['base0C'] },
        { token: 'delimiter.parenthesis', foreground: c['base0C'] },

        // numbers
        { token: 'number', foreground: c['base09'] },
        { token: 'number.float', foreground: c['base09'] },
        { token: 'number.hex', foreground: c['base09'] },

        // green color for comments, because people like to write tutorials in the shaders
        { token: 'comment', foreground: c['base0B'] },
        // gray lines for widget syntax
        { token: 'comment-ui-widget', foreground: c['base03'], fontStyle: 'italic underline' },

        { token: 'glsl-keyword', foreground: c['base0E'] },
        { token: 'glsl-macro', foreground: c['base0E'] },
        { token: 'glsl-precision-modifier', foreground: c['base0E'] },
        { token: 'glsl-predefined-constant', foreground: c['base0D'] },
        { token: 'glsl-operator', foreground: c['base0E'] },
        { token: 'glsl-swizzle', foreground: c['base0B'] },
        { token: 'glsl-storage-modifier', foreground: c['base08'] },

        // float, vec3, mat4
        { token: 'glsl-simple-type', foreground: c['base0A'] },
        { token: 'glsl-sampler-variable', foreground: c['base0A'] },
        { token: 'glsl-image-variable', foreground: c['base0A'] },
        { token: 'glsl-vector-constructor', foreground: c['base0A'] },
        { token: 'glsl-matrix-constructor', foreground: c['base0A'] },

        { token: 'glsl-builtin-function', foreground: c['base0D'] },

        // gl_Position
        { token: 'glsl-reserved-variable', foreground: c['base0D'] },

        // asm
        { token: 'glsl-illegal', foreground: 'ff0000' },
        { token: 'glsl-invalid', foreground: 'ff0000' },

        // visual sugar for shader coding customs

        // iTime, iResolution, etc.
        { token: 'shadertoy-variable', foreground: c['base0D'], fontStyle: 'bold' },
        // g_camera_pos, i_resolution
        { token: 'global-variable', foreground: c['base0D'], fontStyle: 'italic' },
        // uppercase globals (PI, MAX_DIST), or struct types (Ray)
        { token: 'upcase-name', foreground: c['base0D'], fontStyle: 'italic' },
    ],
};

