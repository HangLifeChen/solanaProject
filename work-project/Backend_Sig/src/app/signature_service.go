package main

import(
	"bytes"
	"crypto/ed25519"
	"encoding/base64"
	"encoding/binary"
	"github.com/mr-tron/base58"
	"mylib/src/public"
	"github.com/gagliardetto/solana-go"
)

//var wallet *solana.Wallet
var signer_key		ed25519.PrivateKey
var signer_pubkey	string

func u64ToBytesLE(v uint64) []byte {
	buf := new(bytes.Buffer)
	binary.Write(buf, binary.LittleEndian, v)
	return buf.Bytes()
}

type SignedMessageResponse struct {
	Signature	string	`json:"signature"` // base64 encoded
	Nonce		uint64	`json:"nonce"`
	Timestamp	uint64	`json:"timestamp"`
	Signer		string	`json:"signer"`
}


func signatrue_mint_esneb(signer_bs58 string, amount uint64, nonce uint64)(interface{}, bool){
	timestamp	:= uint64(public.Now_Time_S())

	pubkey, err := solana.PublicKeyFromBase58(signer_bs58)
	if err != nil {
	    public.DBG_ERR("signer bs58 no exist. err[", err, "] signer[", signer_bs58, "]")
	    return "signer bs58 no exist.", false
	}

	message := make([]byte, 56)
	copy(message[:32], pubkey.Bytes())				// signer pubkey
	copy(message[32:40], u64ToBytesLE(amount))		// amount
	copy(message[40:48], u64ToBytesLE(nonce))		// nonce
	copy(message[48:56], u64ToBytesLE(timestamp))	// timestamp

	// sign this message
	signature := ed25519.Sign(signer_key, message)

	public.DBG_LOG("sig[",signature, "]")

	// constract return
	resp := SignedMessageResponse{
		Signature	: base64.StdEncoding.EncodeToString(signature),
		Nonce		: nonce,
		Timestamp	: timestamp,
		Signer		: signer_pubkey,
	}

	return resp, true
}

func init(){
	
	// wallet, _ = solana.WalletFromPrivateKeyBase58(public.Config.Privatekey)

	// public.DBG_LOG("Public Key:", wallet.PublicKey().String())

	// public.DBG_LOG(wallet)

	keyBytes, _ := solana.PrivateKeyFromBase58(public.Config.Privatekey)
	signer_key = ed25519.PrivateKey(keyBytes)


	public_key := signer_key.Public().(ed25519.PublicKey)
	publickey_base58 := base58.Encode(public_key)
	
	signer_pubkey = publickey_base58
}

