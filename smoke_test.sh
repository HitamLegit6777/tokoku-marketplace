#!/usr/bin/env bash
# End-to-end smoke/regression test. Assumes a fresh test server on $BASE.
set -u
BASE="${BASE:-http://localhost:8080}"
ADMIN_USER="${ADMIN_USER:-admin}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:?Set ADMIN_PASSWORD for the test server}"
TMP="$(mktemp -d)"; CUSTOMER="$TMP/customer.cookies"; ADMIN="$TMP/admin.cookies"
PASS=0; FAIL=0
trap 'rm -rf "$TMP"' EXIT
red(){ printf '\033[31m%s\033[0m\n' "$1"; }
grn(){ printf '\033[32m%s\033[0m\n' "$1"; }
check(){ if printf '%s' "$3" | grep -qF -- "$2"; then grn "PASS: $1"; PASS=$((PASS+1)); else red "FAIL: $1 (missing: $2)"; FAIL=$((FAIL+1)); fi; }
checkcode(){ local got; got=$(curl -s -o /dev/null -w '%{http_code}' "${@:4}" "$2"); if [ "$got" = "$3" ]; then grn "PASS: $1 ($got)"; PASS=$((PASS+1)); else red "FAIL: $1 ($got != $3)"; FAIL=$((FAIL+1)); fi; }

echo '== Public + SEO =='
HOME=$(curl -s "$BASE/")
check 'home product grid' 'product-grid' "$HOME"
check 'home theme CSS' '--primary:' "$HOME"
check 'canonical' 'rel="canonical"' "$HOME"
check 'Open Graph' 'property="og:site_name"' "$HOME"
check 'Twitter card' 'name="twitter:card"' "$HOME"
check 'Store JSON-LD' '"@type": "Store"' "$HOME"
checkcode products "$BASE/products" 200
checkcode cart "$BASE/cart" 200
checkcode track "$BASE/track" 200
checkcode robots "$BASE/robots.txt" 200
checkcode sitemap "$BASE/sitemap.xml" 200
checkcode '404 status' "$BASE/nonexistent-xyz" 404
ROBOTS=$(curl -s "$BASE/robots.txt"); check 'robots blocks admin' 'Disallow: /admin' "$ROBOTS"; check 'robots links sitemap' '/sitemap.xml' "$ROBOTS"
SLUG=$(printf '%s' "$HOME" | grep -o '/product/[a-z0-9-]*' | head -1 | sed 's#/product/##')
PD=$(curl -s "$BASE/product/$SLUG")
check 'product renders' 'pd-name' "$PD"; check 'Product JSON-LD' '"@type": "Product"' "$PD"; check 'Offer JSON-LD' '"@type": "Offer"' "$PD"

# Security headers and private noindex
HEADERS=$(curl -sD - -o /dev/null "$BASE/" | tr -d '\r')
check nosniff 'x-content-type-options: nosniff' "$(printf '%s' "$HEADERS" | tr A-Z a-z)"
check frame-deny 'x-frame-options: deny' "$(printf '%s' "$HEADERS" | tr A-Z a-z)"
check 'cart noindex' 'noindex, follow' "$(curl -s "$BASE/cart")"
check 'track noindex' 'noindex, nofollow' "$(curl -s "$BASE/track")"

# Setup is closed after first setup
checkcode 'setup locked GET' "$BASE/setup" 303
checkcode 'setup locked POST' "$BASE/setup" 403 -X POST -d 'store_name=Hacked'
checkcode 'cross-site POST blocked' "$BASE/cart/add" 403 -H 'Sec-Fetch-Site: cross-site' -X POST -d 'product_id=1'

echo '== Cart + checkout =='
curl -s -c "$CUSTOMER" -b "$CUSTOMER" -X POST "$BASE/cart/add" --data 'product_id=1&quantity=2&redirect=/cart' -o /dev/null
CART=$(curl -s -c "$CUSTOMER" -b "$CUSTOMER" "$BASE/cart")
check 'cart item' 'cart-item' "$CART"; check 'cart count' '"count":2' "$(curl -s -c "$CUSTOMER" -b "$CUSTOMER" "$BASE/cart/count")"
# Open redirect must be rejected
LOC=$(curl -s -c "$CUSTOMER" -b "$CUSTOMER" -X POST "$BASE/cart/add" --data 'product_id=1&quantity=1&redirect=//evil.example' -D - -o /dev/null | tr -d '\r' | awk 'tolower($1)=="location:"{print $2}')
check 'open redirect blocked' '/cart' "$LOC"
# Forged payment must be rejected
LOC=$(curl -s -c "$CUSTOMER" -b "$CUSTOMER" -X POST "$BASE/checkout" --data 'customer_name=Test User&customer_phone=08123456789&shipping_address=Jl Test&payment_method=bogus' -D - -o /dev/null | tr -d '\r' | awk 'tolower($1)=="location:"{print $2}')
check 'forged payment blocked' '/checkout?error=payment' "$LOC"
# Normal order
LOC=$(curl -s -c "$CUSTOMER" -b "$CUSTOMER" -X POST "$BASE/checkout" --data 'customer_name=Test User&customer_phone=08123456789&shipping_address=Jl Test&payment_method=transfer' -D - -o /dev/null | tr -d '\r' | awk 'tolower($1)=="location:"{print $2}')
check 'order redirect' '/order/INV-' "$LOC"
if printf '%s' "$LOC" | grep -Eq '^/order/INV-[0-9]{8}-[A-F0-9]{32}$'; then grn 'PASS: 128-bit order id'; PASS=$((PASS+1)); else red "FAIL: 128-bit order id ($LOC)"; FAIL=$((FAIL+1)); fi
OC=$(curl -s "$BASE$LOC"); check 'order confirmation' 'Pesanan Berhasil' "$OC"; check 'order noindex' 'noindex, nofollow' "$OC"

echo '== Admin =='
checkcode 'admin logged-out redirect' "$BASE/admin" 303
LOGIN=$(curl -s "$BASE/admin/login"); check login 'Masuk ke panel admin' "$LOGIN"; check 'admin noindex' 'noindex, nofollow' "$LOGIN"
curl -s -c "$ADMIN" -b "$ADMIN" -X POST "$BASE/admin/login" --data "username=$ADMIN_USER&password=$ADMIN_PASSWORD" -o /dev/null
DASH=$(curl -s -c "$ADMIN" -b "$ADMIN" "$BASE/admin"); check dashboard 'Total Pendapatan' "$DASH"
check 'finance page' 'Laporan Keuangan' "$(curl -s -c "$ADMIN" -b "$ADMIN" "$BASE/admin/finance")"
# Reject active SVG and invalid status
BAD=$(printf '<svg onload="alert(1)"></svg>' | curl -s -c "$ADMIN" -b "$ADMIN" -F 'image=@-;filename=x.svg;type=image/svg+xml' "$BASE/admin/upload")
check 'active SVG rejected' '"success":false' "$BAD"
for p in products categories orders banners coupons pages payment settings appearance finance; do checkcode "admin/$p" "$BASE/admin/$p" 200 -c "$ADMIN" -b "$ADMIN"; done
# Product CRUD
curl -s -c "$ADMIN" -b "$ADMIN" -X POST "$BASE/admin/products/save" --data 'name=Produk Test Otomatis&price=12345&stock=10&is_active=on&track_stock=on&category_id=0' -o /dev/null
check 'admin product created' 'Produk Test Otomatis' "$(curl -s -c "$ADMIN" -b "$ADMIN" "$BASE/admin/products")"
check 'store product visible' 'Produk Test Otomatis' "$(curl -s "$BASE/products?sort=newest")"
# Logout only POST
checkcode 'logout GET disabled' "$BASE/admin/logout" 405 -c "$ADMIN" -b "$ADMIN"
checkcode 'logout POST' "$BASE/admin/logout" 303 -c "$ADMIN" -b "$ADMIN" -X POST

echo; echo "RESULT: $PASS passed, $FAIL failed"; [ "$FAIL" -eq 0 ]
