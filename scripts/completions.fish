complete -c zipcrawl -n "not __fish_seen_subcommand_from tree t cat bat list ls l find fd f grep g x exec diff d completions" -a "tree t cat bat list ls l find fd f grep g x exec diff d completions"
complete -c zipcrawl -n "__fish_seen_subcommand_from cat bat x exec" -f -a "(zipcrawl (string replace -r '^~' \$HOME -- (commandline -opc)[2]) list 2>/dev/null | string replace -r '^\S+\s+' '')"
