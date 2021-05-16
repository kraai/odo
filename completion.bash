# Copyright 2021 Matthew James Kraai
#
# This file is part of odo.
#
# odo is free software: you can redistribute it and/or modify it under the terms of the GNU Affero
# General Public License as published by the Free Software Foundation, either version 3 of the
# License, or (at your option) any later version.
#
# odo is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the
# implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero
# General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License along with odo.  If not,
# see <https://www.gnu.org/licenses/>.

_actions()
{
    local IFS=$'\n'
    COMPREPLY=($(compgen -W '$(odo action ls)' -- "$cur"))
    COMPREPLY=(${COMPREPLY[@]// /\\ })
}

_goals()
{
    local IFS=$'\n'
    COMPREPLY=($(compgen -W '$(odo goal ls)' -- "$cur"))
    COMPREPLY=(${COMPREPLY[@]// /\\ })
}

_odo()
{
    local cur prev words cword
    _init_completion -s || return

    if [[ $cword == 1 ]]; then
	COMPREPLY=($(compgen -W 'action goal' -- "$cur"))
    else
	case ${words[1]} in
	    action)
		if [[ $cword == 2 ]]; then
		    COMPREPLY=($(compgen -W 'add ls rm set' -- "$cur"))
		else
		    case ${words[2]} in
			rm)
			    if [[ $cword == 3 ]]; then
				_actions
			    fi
			    ;;
			set)
			    if [[ $cword == 3 ]]; then
				COMPREPLY=($(compgen -W 'description' -- "$cur"))
			    elif [[ $cword = 4 ]]; then
				_actions
			    fi
			    ;;
		    esac
		fi
		;;
	    goal)
		if [[ $cword == 2 ]]; then
		    COMPREPLY=($(compgen -W 'add ls rm set unset' -- "$cur"))
		else
		    case ${words[2]} in
			add)
			    if [[ $cur == -* ]]; then
				COMPREPLY=($(compgen -W '--action' -- "$cur"))
			    else
				case $prev in
				    --action)
					_actions
					;;
				esac
			    fi
			    ;;
			ls)
			    if [[ $cur == -* ]]; then
				COMPREPLY=($(compgen -W '--all' -- "$cur"))
			    fi
			    ;;
			rm)
			    if [[ $cword == 3 ]]; then
				_goals
			    fi
			    ;;
			set)
			    if [[ $cword == 3 ]]; then
				COMPREPLY=($(compgen -W 'action description' -- "$cur"))
			    else
				case ${words[3]} in
				    action)
					if [[ $cword == 4 ]]; then
					    _goals
					elif [[ $cword == 5 ]]; then
					    _actions
					fi
					;;
				    description)
					if [[ $cword == 4 ]]; then
					    _goals
					fi
					;;
				esac
			    fi
			    ;;
			unset)
			    if [[ $cword == 3 ]]; then
				COMPREPLY=($(compgen -W 'action' -- "$cur"))
			    else
				if [[ $cword == 4 ]]; then
				    _goals
				fi
			    fi
			    ;;
		    esac
		fi
		;;
	esac
    fi
} &&
    complete -F _odo odo
