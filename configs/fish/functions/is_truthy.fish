function is_truthy --argument-names val --description "Check if a value is truthy (non-empty, not 0/false/no)"
    test -n "$val" -a "$val" != "0" -a "$val" != "false" -a "$val" != "no"
end
