cargo b && mkdir lua -p && mv target/debug/libsilicon.so lua/silicon.so -fn
# nvim
set_rtp=":set rtp+=$PWD"
cmd="
:lua require'silicon'
"
RUST_BACKTRACE=1 nvim -u NONE --headless +"$set_rtp" +"$cmd" +quit
