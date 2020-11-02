- find better way to see if we own key to decrypt secrets (can_decrypt) (https://stackoverflow.com/q/64633736/1000145)

Next feature:
- sync filesystem after clearing secret from disk
- edit: do not store secret (temporarily) on disk, store on ramdisk instead (/dev/shm)
- do we have panic protection for secure types (such as `lib::types::Plaintext`)
- generate pass to stdout with --stdout
- copy specific property instead of first line
- sync init set branch
- gtk copy client
- check which gpgme version to require (require_gpgme_version macro)

Ideas:
- annotate commits, to allow easy filtering later on, useful to filter things
  like re-crypt commits if you'd ever want to figure out what time a password
  was last changed at, annotate in ways like: [prs:recrypt]
