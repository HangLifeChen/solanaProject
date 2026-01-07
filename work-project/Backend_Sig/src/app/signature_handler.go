package main

import (
	"mylib/src/public"
)

func get_signatrue(params map[string]string) (interface{}, bool) {

	amount_str := params["amount"]
	signer := params["signer"]
	nonce_str := params["nonce"]

	amount := uint64(public.ConvertStrToNum(amount_str))
	nonce := uint64(public.ConvertStrToNum(nonce_str))

	return signatrue_mint_esneb(signer, amount, nonce)
}
