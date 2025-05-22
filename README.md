# i8086仮想マシン

## テスト方法

```sh
./scripts/test.sh 1.c
```


3.c
mmvm
0337: 8d9ee7fb      lea bx, [bp-419]

自分の実装
0337: 8d9ee7fb      LEA BX, [BP+fbe7]

mod=10の時、disp=disp high; disp low

8d 9e e7 fb
10001101 10011110 11100111 11111011

mod=10
reg=111
r/m=110

disp: u16 = 0xfbe7
disp: i16 = 0x-419
