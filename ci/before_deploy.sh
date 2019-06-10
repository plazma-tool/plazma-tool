echo "=== before_deploy.sh ==="

os=$TRAVIS_OS_NAME
echo "OS: '$os'"

if [[ "$os" == "linux" ]]; then
	cp ./plazma/target/release/plazma ./plazma_linux
elif [[ "$os" == "osx" ]]; then
	cp ./plazma/target/release/plazma ./plazma_osx
fi
