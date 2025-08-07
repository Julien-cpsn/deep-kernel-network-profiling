nm -D /usr/lib/x86_64-linux-gnu/"$1" | grep -i "$2" | sort

#ex: ./find_lib_function.sh librte_eal.so malloc