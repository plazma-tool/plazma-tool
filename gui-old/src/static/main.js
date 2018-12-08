require.config({
    paths: {
        'vs': '/static/assets/vendor/node_modules/monaco-editor/min/vs',
        'monaco-vim': '/static/assets/vendor/node_modules/monaco-vim/dist/monaco-vim',
    }
});

monaco.languages.register({ id: 'glsl' });

monaco.languages.setMonarchTokensProvider('glsl', {
    //"comments": {
    //    "lineComment": "//",
    //    "blockComment": [ "/*", "*/" ]
    //},
    //"brackets": [
    //    ["{", "}"],
    //    ["[", "]"],
    //    ["(", ")"]
    //],
    tokenizer: {
        root: []
    }
});

const default_shader = [
    '#version 430',
    '',
    'in vec2 texCoord;',
    'out vec4 out_color;',
    '',
    'layout(location = 0) uniform float iTime;',
    'layout(location = 1) uniform vec2 iResolution;',
    'layout(location = 3) uniform vec2 screenResolution;',
    '',
    '// --- tool ---',
    '',
    'void mainImage( out vec4 fragColor, in vec2 fragCoord )',
    '{',
    '  // Normalized pixel coordinates (from 0 to 1)',
    '  vec2 uv = fragCoord/iResolution.xy;',
    '',
    '  // Time varying pixel color',
    '  vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));',
    '',
    '  // Output to screen',
    '  fragColor = vec4(col,1.0);',
    '}',
    '',
    '// --- tool ---',
    '',
    'void main() {',
    '  vec4 col = vec4(0.0, 0.0, 0.0, 1.0);',
    '  mainImage(col, gl_FragCoord.xy);',
    '  out_color = col;',
    '}',
].join('\n');

var editor;
var vimMode;
var monacoVim;

var containerNode = document.getElementById('container');
var statusNode = document.getElementById('status');

var sentUpdateSinceChange = false;

// Create WebSocket connection.
const socket = new WebSocket('ws://localhost:8080/ws/');

socket.addEventListener('open', function (event) {
    var msg = {
        data_type: 'SetFragmentShader',
        data: default_shader,
    };
    socket.send(JSON.stringify(msg));
});

socket.addEventListener('message', function (event) {
    var msg = JSON.parse(event.data);
    if (msg.data_type === 'SetFragmentShader') {
        editor.getModel().setValue(JSON.parse(msg.data));
        sentUpdateSinceChange = true;
    }
});

// Setup Monaco editor.
require(['vs/editor/editor.main', 'monaco-vim'], function(a, monaco_vim) {
    editor = monaco.editor.create(containerNode, {
        value: default_shader,
        language: 'glsl',
        lineNumbers: "on",
        roundedSelection: false,
        scrollBeyondLastLine: true,
        theme: "vs-dark",
    });

    monacoVim = monaco_vim;
    vimMode = monaco_vim.initVimMode(editor, statusNode);

    editor.focus();
    editor.setPosition({ lineNumber: 1, column: 1 });

    const initialVersion = editor.getModel().getAlternativeVersionId();
    let currentVersion = initialVersion;
    let lastVersion = initialVersion;

    editor.onDidChangeModelContent(e => {
        sentUpdateSinceChange = false;

        const versionId = editor.getModel().getAlternativeVersionId();
        // undoing
        if (versionId < currentVersion) {
            enableRedoButton();
            // no more undo possible
            if (versionId === initialVersion) {
                disableUndoButton();
            }
        } else {
            // redoing
            if (versionId <= lastVersion) {
                // redoing the last change
                if (versionId == lastVersion) {
                    disableRedoButton();
                }
            } else { // adding new change, disable redo when adding new changes
                disableRedoButton();
                if (currentVersion > lastVersion) {
                    lastVersion = currentVersion;
                }
            }
            enableUndoButton();
        }
        currentVersion = versionId;
    });

});

function undo() {
    editor.trigger('aaaa', 'undo', 'aaaa');
    editor.focus();
}

function redo() {
    editor.trigger('aaaa', 'redo', 'aaaa');
    editor.focus();
}

function enableUndoButton() {
    document.getElementById("undoButton").disabled = false;
}

function disableUndoButton() {
    document.getElementById("undoButton").disabled = true;
}

function enableRedoButton() {
    document.getElementById("redoButton").disabled = false;
}

function disableRedoButton() {
    document.getElementById("redoButton").disabled = true;
}

function toggleVimMode() {
    var node = document.getElementById("vim-enabled");
    if (node.checked) {
        vimMode = monacoVim.initVimMode(editor, statusNode);
    } else {
        vimMode.dispose();
        statusNode.innerHTML = '';
    }
    // FIXME editor doesn't regain focus
    editor.focus();
}

document.getElementById("vim-enabled").onchange = function() { toggleVimMode(); };

function sendSetFragmentShader() {
    if (sentUpdateSinceChange) {
        return;
    } else {
        var msg = {
            data_type: 'SetFragmentShader',
            data: editor.getModel().getValue(),
        };
        socket.send(JSON.stringify(msg));
        sentUpdateSinceChange = true;
    }
}

var timeoutID = window.setInterval(sendSetFragmentShader, 1000);

