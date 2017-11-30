# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # DONE Update this to build the artifacts that matter to you
    cross rustc --bin just --target $TARGET --release -- -C lto

    # DONE Update this to package the right artifacts
    cp target/$TARGET/release/just $stage/
    cp GRAMMAR.md $stage/
    cp LICENSE $stage/
    cp README.asc $stage/

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
