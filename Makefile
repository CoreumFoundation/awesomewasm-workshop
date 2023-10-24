COREUM_CHAIN_ID="coreum-devnet-1"
COREUM_DENOM=udevcore
COREUM_NODE=http://localhost:26657
COREUM_VERSION="{Cored version}"
COREUM_CHAIN_ID_ARGS=--chain-id=$(COREUM_CHAIN_ID)
COREUM_NODE_ARGS=--node=$(COREUM_NODE)
COREUM_HOME=$(HOME)/.core/"$(COREUM_CHAIN_ID)"
COREUM_BINARY_NAME=$(shell arch | sed s/aarch64/cored-linux-arm64/ | sed s/x86_64/cored-linux-amd64/)

DEV_WALLET=dev-wallet
CODE_ID=1
SUBUNIT=gil

# replace this after you instantiate your contract
_CONTRACT_ADDRESS_=devcore14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sd4f0ak
dev:
	echo "${PWD}"
	echo `basename "${PWD}"`
	cargo build
test:
	cargo test -- --nocapture
add_account:
	cored-00 keys add ${DEV_WALLET} --recover
build:
	docker run --rm -v "${PWD}":/code --mount type=volume,source=`basename "${PWD}"`_cache,target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/rust-optimizer:0.12.6

deploy:
	RES=$$(cored-00 tx wasm store artifacts/ft_airdrop.wasm --from ${DEV_WALLET} --gas auto --gas-adjustment 1.3 -y -b block --output json "$(COREUM_NODE_ARGS)" "$(COREUM_CHAIN_ID_ARGS)") ; \
	echo $$RES ; \
	CODE_ID=$$(echo $$RES | jq -r '.logs[0].events[-1].attributes[-1].value') ; \
	echo "Code ID: $$CODE_ID"
check:
	cored-00 q wasm code-info $(CODE_ID) $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)
keys:
	cored-00 keys list
q:
	cored-00 q bank balances devcore1pv6jk3k89uqly098ytnpfkehq6hp35jzmyfk4y
fund:
	cored-00 tx bank send alice devcore10hek368msx93z0medf6ylzffwn38myckprznau 10000000udevcore --broadcast-mode=block
instantiate:
	cored-00 tx wasm instantiate $(CODE_ID) \
	"{\"symbol\":\"AWESOMECOIN\",\"subunit\":\"$(SUBUNIT)\",\"precision\":6,\"initial_amount\":\"10000000\",\"airdrop_amount\":\"100000\"}" \
	--amount="10000000$(COREUM_DENOM)" --no-admin --label "awesomwasm token" --from ${DEV_WALLET} --gas auto --gas-adjustment 1.3 -b block -y $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)
contract_address:
	cored-00 q wasm list-contract-by-code $(CODE_ID) --output json $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS) | jq -r '.contracts[-1]'
	CONTRACT_ADDRESS=$(shell cored-00 q wasm list-contract-by-code $(CODE_ID) --output json $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS) | jq -r '.contracts[-1]')
	echo $$CONTRACT_ADDRESS
denom:
	cored-00 q wasm list-contract-by-code $(CODE_ID) --output json $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS) | jq -r '.contracts[-1]'
	CONTRACT_ADDRESS=$(shell cored-00 q wasm list-contract-by-code $(CODE_ID) --output json $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS) | jq -r '.contracts[-1]')
	echo $$CONTRACT_ADDRESS;
	FT_DENOM=$$SUBUNIT-$$CONTRACT_ADDRESS
	echo $$FT_DENOM;
	cored-00 q bank total --denom $(FT_DENOM) $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)
token_total_supply:
	cored-00 q bank total --denom gil-$(_CONTRACT_ADDRESS_) --node=http://localhost:26657 --chain-id=coreum-devnet-1
token_metadata:
	cored-00 q bank denom-metadata --denom gil-$(_CONTRACT_ADDRESS_) --node=http://localhost:26657 --chain-id=coreum-devnet-1
asset_ft:
	cored-00 q assetft token gil-$(_CONTRACT_ADDRESS_) --node=http://localhost:26657 --chain-id=coreum-devnet-1
mint_for_airdrop:
	cored-00 tx wasm execute devcore14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sd4f0ak "{\"mint_for_airdrop\":{\"amount\":\"5000000\" }}" --from ${DEV_WALLET} -b block -y $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)
receive_airdrop:
	cored-00 tx wasm execute $(_CONTRACT_ADDRESS_) '{"receive_airdrop":{}}' --from ${DEV_WALLET} -b block -y $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)
balances:
	cored-00 q bank balances $(shell cored-00 keys show ${DEV_WALLET} $(COREUM_CHAIN_ID_ARGS) -a) --denom gil-devcore1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts79je4k $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)
airdrop_amount:
	cored-00 q wasm contract-state smart $(_CONTRACT_ADDRESS_) '{"minted_for_airdrop": {}}' $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)
commission_test:
	cored-00 tx bank send $(shell cored-00 keys show dev-wallet $(COREUM_CHAIN_ID_ARGS) -a) $(shell cored-00 keys show alice $(COREUM_CHAIN_ID_ARGS) -a) 1000gil-$(_CONTRACT_ADDRESS_) --from dev-wallet -b block -y $(COREUM_NODE_ARGS) $(COREUM_CHAIN_ID_ARGS)

## content?
# smart contract vs smart token
# original idea for smart token
# graphic on how many erc20/721, and duplication of code, and also trust of said code working
# smart tokens allow you to instantiate a token that follows a spec

# the fundamental supported capabilities of tokens so far (burning, freezing, minting, etc)
# current development tasks (IBC about to launch, several bridges, we have an XRP bridge)
# in this generation many mainstream chains are being built on a chain-development framework (cosmos, subastrate)
# fundamental behavior
# can extend the behavior of the asset using WASM



# 1
# - amount: "100000"
#   denom: gil-devcore1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts79je4k
# - amount: "72559646"
#   denom: udevcore
# 2
# - amount: "97800"
#   denom: gil-devcore1yw4xvtc43me9scqfr2jr2gzvcxd3a9y4eq7gaukreugw2yd2f8ts79je4k
# - amount: "72534646"
#   denom: udevcore
# pagination:

# q_wasm:
# 	cored-00 q wasm contract-state smart $CONTRACT_ADDRESS '{"minted_for_airdrop": {}}' $COREUM_NODE_ARGS $COREUM_CHAIN_ID_ARGS

# contract_balance:
# 	cored-00 q bank balances $(cored-00 keys show dev-test $COREUM_CHAIN_ID_ARGS -a) --denom $FT_DENOM $COREUM_NODE_ARGS $COREUM_CHAIN_ID_ARGS

# execute:
# 	cored-00 tx wasm execute $CONTRACT_ADDRESS '{"receive_airdrop":{}}' --from dev-test -b block -y $COREUM_NODE_ARGS $COREUM_CHAIN_ID_ARGS


# # cored-00 q wasm list-contract-by-code $CODE_ID --output json $COREUM_NODE_ARGS $COREUM_CHAIN_ID_ARGS
# # CONTRACT_ADDRESS=$(cored-00 q wasm list-contract-by-code $CODE_ID --output json $COREUM_NODE_ARGS $COREUM_CHAIN_ID_ARGS | jq -r '.contracts[-1]')
# # echo "Contract address: $CONTRACT_ADDRESS"

# balance:
# 	cored-00 q bank balances devcore10hek368msx93z0medf6ylzffwn38myckprznau

# instantiate:
# 	cored-00 tx wasm instantiate $CODE_ID \
#  	"{\"symbol\":\"mysymbol\",\"subunit\":\"$SUBUNIT\",\"precision\":6,\"initial_amount\":\"1000000000\",\"airdrop_amount\":\"1000000\"}" \
#   	--amount="10000000udevcore" --no-admin --label "My smart token" --from dev-test --gas auto --gas-adjustment 1.3 -b block -y $COREUM_NODE_ARGS $COREUM_CHAIN_ID_ARGS

# fund:
# 	cored-00 tx bank send alice devcore10hek368msx93z0medf6ylzffwn38myckprznau 10000000udevcore --broadcast-mode=block
# view_code:
# 	cored-00 q wasm code-info $CODE_ID $COREUM_NODE_ARGS $COREUM_CHAIN_ID_ARGS  

# deploy2:
# 	RES=$(cored-00 tx wasm store artifacts/ft_airdrop.wasm \                                                       
#     	--from dev-test --gas auto --gas-adjustment 1.3 -y -b block --output json $COREUM_NODE_ARGS $COREUM_CHAIN_ID_ARGS)
# 	echo $RES
# 	CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[-1].value')
# 	echo "Code ID: $CODE_ID";
# list:
# 	cored-00 keys list   

