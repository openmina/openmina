const TerserPlugin = require("terser-webpack-plugin");
console.log('Loading custom webpack!')
module.exports = {
  experiments: {
    topLevelAwait: true,
  },
  optimization: {
    minimize: true,
    minimizer: [
      new TerserPlugin({
        terserOptions: {
          ecma: undefined,
          parse: {},
          compress: {
            keep_classnames: true,
            keep_fargs: true,
            keep_fnames: true,
          },
          keep_classnames: true,
          keep_fnames: true,
        },
      }),
    ],
  }
}
