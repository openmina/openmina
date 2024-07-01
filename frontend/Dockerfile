FROM node:18 AS BUILD_IMAGE
ARG BUILD_CONFIGURATION=producer
WORKDIR /app
COPY . .
RUN npm install
RUN node_modules/.bin/ng build --configuration ${BUILD_CONFIGURATION}
RUN npm prune --production

RUN echo "---------- USING APACHE ----------"

FROM httpd:2.4
COPY --from=BUILD_IMAGE /app/dist/frontend /usr/local/apache2/htdocs/
COPY --from=BUILD_IMAGE /app/httpd.conf /usr/local/apache2/conf/httpd.conf
#FROM nginx:alpine
#COPY --from=BUILD_IMAGE /app/dist/frontend /usr/share/nginx/html
#COPY --from=BUILD_IMAGE /app/nginx.conf /etc/nginx/nginx.conf
