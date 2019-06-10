echo "=== before_script.sh ==="

export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

nvm use 10

node --version
npm --version
yarn --version
yarn cache dir

cd gui
yarn install
npm run build

