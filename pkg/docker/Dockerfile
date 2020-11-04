FROM alpine:latest
LABEL maintainer="Tim Visée <3a4fb3964f@sinenomine.email>"

RUN apk --no-cache add git gnupg openssh-client

COPY ./prs /

WORKDIR /root/
ENTRYPOINT ["/prs"]
