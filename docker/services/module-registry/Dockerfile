FROM golang:1.18-alpine

WORKDIR /app

COPY . .

RUN go mod download
RUN go build -o module-registry

EXPOSE 8080

CMD ["./module-registry"]
