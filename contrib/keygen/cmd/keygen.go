package main

import (
	"crypto/ed25519"
	"encoding/hex"
	"flag"
	"fmt"
	"log"
	"os"
)

type PasetoV2PublicKey struct {
	material ed25519.PublicKey
}

type PasetoV2SecretKey struct {
	material ed25519.PrivateKey
}

func NewPasetoV2SecretKey() PasetoV2SecretKey {
	_, privKey, err := ed25519.GenerateKey(nil)
	if err != nil {
		panic("fuck")
	}
	return PasetoV2SecretKey{privKey}
}

func (k PasetoV2SecretKey) Public() PasetoV2PublicKey {
	material, ok := k.material.Public().(ed25519.PublicKey)
	if !ok {
		panic("public key wrong")
	}
	return PasetoV2PublicKey{material}
}

func (k PasetoV2PublicKey) ExportHex() string {
	return hex.EncodeToString(k.material)
}

func (k PasetoV2SecretKey) ExportHex() string {
	return hex.EncodeToString(k.material)
}

var (
	outFile = flag.String("o", "", "The file to export to. Prints out otherwise.")
)

func main() {
	flag.Parse()
	_priv := NewPasetoV2SecretKey()
	pub := _priv.Public().ExportHex()
	priv := _priv.ExportHex()
	if *outFile == "" {
		fmt.Printf("Public: %s\nPrivate: %s\n", pub, priv)
	} else {
		f, err := os.Create(*outFile)
		if err != nil {
			log.Fatal(err)
		}
		fmt.Fprintf(f, "Public: %s\nPrivate: %s\n", pub, priv)
	}
}
