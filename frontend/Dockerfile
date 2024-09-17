FROM node:18 AS BUILD_IMAGE
# Doesn't matter what we put here - it get's overwritten by the docker build command
ARG BUILD_CONFIGURATION=production
WORKDIR /app
COPY . .
RUN npm install
RUN node_modules/.bin/ng build --configuration ${BUILD_CONFIGURATION}
RUN npm prune --production

RUN echo "---------- USING APACHE ----------"

FROM httpd:2.4
COPY --from=BUILD_IMAGE /app/dist/frontend /usr/local/apache2/htdocs/
COPY --from=BUILD_IMAGE /app/httpd.conf /usr/local/apache2/conf/httpd.conf

COPY docker/startup.sh /usr/local/bin/startup.sh
RUN chmod +x /usr/local/bin/startup.sh

ENTRYPOINT ["/usr/local/bin/startup.sh"]

CMD ["httpd-foreground"]