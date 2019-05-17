const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin');

module.exports = {
    webpack: {
        plugins: [
            new MonacoWebpackPlugin()
        ],
		// FIXME js-yaml-loader
        //module: {
        //    rules: [{
        //        test: /\.ya?ml$/,
        //        use: 'js-yaml-loader',
        //    }]
        //},
    },
};

