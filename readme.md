# ConoHa Cert

1. run `cargo build --release`
2. run 
```
sudo certbot certonly \
    -d hoge.example.com \
    --preferred-challenges dns-01 \
    --manual \
    --manual-auth-hook ./auth.sh \
    --manual-cleanup-hook ./clean.sh
```
