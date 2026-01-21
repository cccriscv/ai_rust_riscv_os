# 建立一個 32MB 的空檔案，填滿 0
dd if=/dev/zero of=disk.img bs=1M count=32
echo "Hello from VirtIO Disk!" | dd of=disk.img conv=notrunc
