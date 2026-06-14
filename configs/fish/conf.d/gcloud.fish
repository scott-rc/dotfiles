set -l gcloud_inc (brew --prefix)/share/google-cloud-sdk/path.fish.inc
test -f $gcloud_inc; and source $gcloud_inc
