//go:build !prod
// +build !prod

package main

import (
	"mylib/src/module/route_manager"
	_ "mylib/src/public"
)

// ----------------Global Parameter>

// ----------------Function>

func check_server(headers map[string]string) (map[string]string, bool) {

	if headers["auth"] == "5bnas2iecb7kulg4ai1" {
		return map[string]string{}, true
	}
	return map[string]string{}, false
}

func APP_Entry() {

	route := route_manager.New()

	route.Route_Get("signatrue", get_signatrue).
		ReqLimit(10).
		RecvParams("amount", "signer", "nonce").
		Alert("get signatrue error").
		Middle(check_server).
		MiddleParams("auth").
		MiddleAlert("auth check error")
	route.Init_Route("0.0.0.0:8002")
}
