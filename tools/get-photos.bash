#!/bin/bash

set -euo pipefail

[ $# -eq 2 ] || { echo "Usage: $0 <source-dir> <destination-dir>"; exit 1; }

SRC_DIR=$1
DST_BASE_DIR=$2

function main {
    for i in $(find ${SRC_DIR} -type f); do
        if echo ${i} | grep -iq '\.\(jpg\|cr2\)$'; then
            local create_date=$(exiftool -CreateDate -b "${i}")
            local src_filename=$(basename "${i}")
            local src_extension=$(echo ${src_filename} |tr A-Z a-z |sed -e 's/^.*\.\(jpg\|cr2\)$/\1/')
            local dst_extension=$(echo ${src_extension} |tr A-Z a-z)
            local target_filename=$(echo ${create_date} |tr : - |tr ' ' _).${dst_extension}
            local dst_dir=${DST_BASE_DIR}/$(echo ${target_filename} |cut -d_ -f1)
            local target=${dst_dir}/${target_filename}

            if [ -e "${target}" ]; then
                if cmp -s "${i}" "${target}"; then
                    echo "${i} is already done"
                else
                    echo "${i} and ${target} differ. Giving up!"
                    exit 1
                fi
            else
                echo "Copying ${i} to ${target}"
                mkdir -p "$(dirname ${target})"
                cp "${i}" "${target}"
            fi
        else
            echo ${i}: ???
        fi
    done
}

main
