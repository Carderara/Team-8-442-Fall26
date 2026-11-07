### Method 1 ###

```
sudo date -s "YYYY-MM-DD HH:MM:SS" && echo "YYYY MM DD HH mm SS" | ./target/release/timelock
```

### Method 2 ###
```
echo "2017 01 01 00 00 00" > epoch.txt
sudo date -s "2017-03-23 18:02:06" && ./target/release/timelock < epoch.txt
```
