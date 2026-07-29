#!/usr/bin/env bash
# End-to-end smoke test for TokoKu. Boots nothing itself; assumes server on $BASE.
# Verifies public pages, admin auth, product CRUD, cart, checkout, order, theme switch.
set -u
BASE="${BASE:-http://localhost:8080}"
JAR="$(mktemp)"
PASS=0; FAIL=0
red(){ printf '\033[31m%s\033[0m\n' "$1"; }
grn(){ printf '\033[32m%s\033[0m\n' "$1"; }

check() { # name  expected_substr  actual
  if printf '%s' "$3" | grep -qF -- "$2"; then grn "PASS: $1"; PASS=$((PASS+1));
  else red "FAIL: $1 (missing: $2)"; FAIL=$((FAIL+1)); fi
}
code() { curl -s -o /dev/null -w "%{http_code}" "$@"; }
checkcode() { # name url expected [use_jar]
  local c
  if [ "${4:-}" = "jar" ]; then c=$(curl -s -c "$JAR" -b "$JAR" -o /dev/null -w "%{http_code}" "$2");
  else c=$(code "$2"); fi
  if [ "$c" = "$3" ]; then grn "PASS: $1 ($c)"; PASS=$((PASS+1));
  else red "FAIL: $1 (got $c want $3)"; FAIL=$((FAIL+1)); fi
}

echo "== Public pages =="
HOME=$(curl -s "$BASE/")
check "home renders store name" "Dapur Nusantara" "$HOME"
check "home has product grid" "product-grid" "$HOME"
check "home theme CSS injected" "--primary:" "$HOME"
checkcode "products page" "$BASE/products" 200
checkcode "cart page" "$BASE/cart" 200
checkcode "track page" "$BASE/track" 200
checkcode "404 works" "$BASE/nonexistent-xyz" 404

# Grab a product slug from home
SLUG=$(printf '%s' "$HOME" | grep -o '/product/[a-z0-9-]*' | head -1 | sed 's#/product/##')
echo "Detected product slug: $SLUG"
if [ -n "$SLUG" ]; then
  PD=$(curl -s "$BASE/product/$SLUG")
  check "product detail renders" "pd-name" "$PD"
  check "product detail has add-to-cart" "cart/add" "$PD"
fi

echo "== Category & search =="
checkcode "category page" "$BASE/category/kopi" 200
SR=$(curl -s "$BASE/search?q=kopi")
check "search returns results" "product" "$SR"

echo "== Cart flow =="
# Add product id 1 to cart
curl -s -c "$JAR" -b "$JAR" -X POST "$BASE/cart/add" --data "product_id=1&quantity=2&redirect=/cart" -o /dev/null
CART=$(curl -s -c "$JAR" -b "$JAR" "$BASE/cart")
check "cart shows item" "cart-item" "$CART"
CNT=$(curl -s -c "$JAR" -b "$JAR" "$BASE/cart/count")
check "cart count api" '"count":2' "$CNT"

echo "== Checkout & order =="
CO=$(curl -s -c "$JAR" -b "$JAR" "$BASE/checkout")
check "checkout page renders" "Informasi Pengiriman" "$CO"
# Place order
curl -s -c "$JAR" -b "$JAR" -X POST "$BASE/checkout" \
  --data "customer_name=Test User&customer_phone=08123456789&shipping_address=Jl Test 1&payment_method=transfer" \
  -D /tmp/tk_headers.txt -o /dev/null
ORDER_LOC=$(grep -i '^location:' /tmp/tk_headers.txt | tr -d '\r' | awk '{print $2}')
echo "Order redirect: $ORDER_LOC"
check "order redirect to /order/" "/order/INV-" "$ORDER_LOC"
if [ -n "$ORDER_LOC" ]; then
  OC=$(curl -s "$BASE$ORDER_LOC")
  check "order confirmation renders" "Pesanan Berhasil" "$OC"
  check "order shows bank transfer" "Transfer Bank" "$OC"
fi

echo "== Admin auth =="
checkcode "admin redirects when logged out" "$BASE/admin" 303
LOGIN=$(curl -s "$BASE/admin/login")
check "login page renders" "Masuk ke panel admin" "$LOGIN"
# Login
curl -s -c "$JAR" -b "$JAR" -X POST "$BASE/admin/login" --data "username=admin&password=admin123" -o /dev/null
DASH=$(curl -s -c "$JAR" -b "$JAR" "$BASE/admin")
check "dashboard after login" "Dashboard" "$DASH"
check "dashboard shows stats" "Total Pendapatan" "$DASH"

echo "== Admin product CRUD =="
curl -s -c "$JAR" -b "$JAR" -X POST "$BASE/admin/products/save" \
  --data "name=Produk Test Otomatis&price=12345&stock=10&is_active=on&track_stock=on&category_id=0" -o /dev/null
PLIST=$(curl -s -c "$JAR" -b "$JAR" "$BASE/admin/products")
check "new product appears in admin" "Produk Test Otomatis" "$PLIST"
check "product appears on storefront" "Produk Test Otomatis" "$(curl -s "$BASE/products?sort=newest")"

echo "== Admin orders =="
AO=$(curl -s -c "$JAR" -b "$JAR" "$BASE/admin/orders")
check "admin orders list" "Test User" "$AO"

echo "== Theme switch =="
curl -s -c "$JAR" -b "$JAR" -X POST "$BASE/admin/appearance/theme" --data "theme=ocean" -o /dev/null
HOME2=$(curl -s "$BASE/")
check "theme switched to ocean (blue primary)" "#0ea5e9" "$HOME2"
curl -s -c "$JAR" -b "$JAR" -X POST "$BASE/admin/appearance/theme" --data "theme=coffee" -o /dev/null

echo "== Admin sub-pages =="
for p in categories banners coupons pages payment settings appearance; do
  checkcode "admin/$p" "$BASE/admin/$p" 200 jar
done

echo ""
echo "==================================="
echo "RESULT: $PASS passed, $FAIL failed"
echo "==================================="
rm -f "$JAR"
[ "$FAIL" -eq 0 ]
