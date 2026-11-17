# rAthena built from the local checkout (../../rathena is the build context) for the packet version roweb targets.
FROM alpine:3.23 AS build
RUN apk add --no-cache make gcc g++ zlib-dev mariadb-dev pcre-dev linux-headers bash
ARG PACKETVER=20260219
# The rules the server plays by: "yes" is the classic game (pre-renewal: its damage formulas, its monsters'
# life and experience, `db/pre-re` and `npc/pre-re`), "no" is Renewal.
ARG PRERE=yes
COPY . /rathena
WORKDIR /rathena
# -j4: each map-server translation unit needs close to a gigabyte of RAM.
RUN ./configure --enable-packetver=${PACKETVER} --enable-prere=${PRERE} && make -j4 server

FROM alpine:3.23
RUN apk add --no-cache libstdc++ zlib mariadb-connector-c pcre netcat-openbsd
COPY --from=build /rathena/login-server /rathena/char-server /rathena/map-server /rathena/
COPY --from=build /rathena/conf /rathena/conf
COPY --from=build /rathena/db /rathena/db
COPY --from=build /rathena/npc /rathena/npc
WORKDIR /rathena
