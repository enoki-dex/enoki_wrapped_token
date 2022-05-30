. "$(dirname "$0")"/setup.sh

start "test 1"
assert_eq "$(echo "123" | grep "12")" "123"
end

start "test 2"
assert_eq "abc" "123" # fail here
end

start "test 3"
assert_eq "abc" "abc"
end
