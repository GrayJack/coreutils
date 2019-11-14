runtest () {
    local -r prog="$1"
    local -r test="$2"
    local -r name=`basename $test`
    shift 2

    adir=`mktemp -p /tmp -d runtest-${name%%.sh}-XXXXXX`
    env -C $adir CSPLIT="csplit" bash $test >$adir/stdout.txt 2>$adir/stderr.txt
    echo "exit code: $?" >$adir/exit.txt
 
    bdir=`mktemp -p /tmp -d runtest-${name%%.sh}-XXXXXX`
    env -C $bdir CSPLIT="$prog" bash $test >$bdir/stdout.txt 2>$bdir/stderr.txt
    echo "exit code: $?" >$bdir/exit.txt

    if diff -ur $adir $bdir; then
	echo "$name compare equal, removing directories."
	rm -rf $adir $bdir
    fi
}

csplit=$PWD/${1:?Usage: runall <csplit> <test> ...}
shift
for T in ${@:-test-[1-9]*.sh}; do
    runtest $csplit $PWD/$T 
done

