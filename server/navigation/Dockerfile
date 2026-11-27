# rAthena's map-server-generator, built from the local checkout as ../Dockerfile builds the server (the same
# sources, packet version and rules). `../navigation.sh` runs it for the navigation's tables.
FROM alpine:3.23 AS build
RUN apk add --no-cache make gcc g++ zlib-dev mariadb-dev pcre-dev linux-headers bash
ARG PACKETVER=20260219
ARG PRERE=yes
COPY . /rathena
WORKDIR /rathena
# One after another: `make -j4 tools` builds libconfig twice at once and fails.
RUN ./configure --enable-packetver=${PACKETVER} --enable-prere=${PRERE} \
 && make libconfig && make -j4 rapidyaml yaml-cpp && make -j4 common && make -j4 -C src/map tools

FROM alpine:3.23
RUN apk add --no-cache libstdc++ zlib mariadb-connector-c pcre
COPY --from=build /rathena/map-server-generator /rathena/
COPY --from=build /rathena/conf /rathena/conf
COPY --from=build /rathena/db /rathena/db
COPY --from=build /rathena/npc /rathena/npc
WORKDIR /rathena
