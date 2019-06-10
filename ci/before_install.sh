echo "=== before_install.sh ==="

os=$TRAVIS_OS_NAME
echo "OS: '$os'"

if [[ "$os" == "linux" ]]; then

	touch ~/.bash_profile
	curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.34.0/install.sh | bash
	export NVM_DIR="$HOME/.nvm"
	[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
	command -v nvm
	nvm install 10
	nvm use 10
	npm install -g yarn

	sudo apt-get update
	sudo apt-get install -y webkit2gtk-4.0

elif [[ "$os" == "osx" ]]; then

	touch ~/.bash_profile
	curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.34.0/install.sh | bash
	export NVM_DIR="$HOME/.nvm"
	[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
	command -v nvm
	nvm install 10
	nvm use 10
	npm install -g yarn

	#brew tap astroidmail/homebrew-astroid
	# This takes over 10m. travis_wait is required when calling before_install.sh
	#brew install webkitgtk

fi

