/* Generated with IDL v0.2.0 */

#include "./test.h"

namespace Packet {




const std::optional<std::string>& LoginRequest::get_username() const noexcept {
    return username;
}

LoginRequest& LoginRequest::set_username(const std::optional<std::string>& username) {
    this->username = username;
    return *this;
}

const std::array<char, 32>& LoginRequest::get_password() const noexcept {
    return password;
}

LoginRequest& LoginRequest::set_password(const std::array<char, 32>& password) {
    this->password = password;
    return *this;
}


const std::string& LoginResponse::get_sessionID() const noexcept {
    return std::get<std::string>(__data);
}

LoginResponse& LoginResponse::set_sessionID(const std::string& sessionID) {
    __data = sessionID;
    return *this;
}

std::string& LoginResponse::make_sessionID() {
    std::string tmp;
    set_sessionID(tmp);
    return std::get<std::string>(__data);
}

const LoginError& LoginResponse::get_error() const noexcept {
    return std::get<LoginError>(__data);
}

LoginResponse& LoginResponse::set_error(const LoginError& error) {
    __data = error;
    return *this;
}

LoginError& LoginResponse::make_error() {
    LoginError tmp;
    set_error(tmp);
    return std::get<LoginError>(__data);
}

const std::string_view LoginResponse::selection() const noexcept {
    const size_t index = __data.index();
    switch (index) {
        case 1:
            return "sessionID";
        case 2:
            return "error";
        default:
            return "unselected";
    }
}


const PingRequest& Request::get_ping() const noexcept {
    return std::get<PingRequest>(__data);
}

Request& Request::set_ping(const PingRequest& ping) {
    __data = ping;
    return *this;
}

PingRequest& Request::make_ping() {
    PingRequest tmp;
    set_ping(tmp);
    return std::get<PingRequest>(__data);
}

const LoginRequest& Request::get_login() const noexcept {
    return std::get<LoginRequest>(__data);
}

Request& Request::set_login(const LoginRequest& login) {
    __data = login;
    return *this;
}

LoginRequest& Request::make_login() {
    LoginRequest tmp;
    set_login(tmp);
    return std::get<LoginRequest>(__data);
}

const std::string_view Request::selection() const noexcept {
    const size_t index = __data.index();
    switch (index) {
        case 1:
            return "ping";
        case 2:
            return "login";
        default:
            return "unselected";
    }
}


const PongResponse& Response::get_pong() const noexcept {
    return std::get<PongResponse>(__data);
}

Response& Response::set_pong(const PongResponse& pong) {
    __data = pong;
    return *this;
}

PongResponse& Response::make_pong() {
    PongResponse tmp;
    set_pong(tmp);
    return std::get<PongResponse>(__data);
}

const LoginResponse& Response::get_login() const noexcept {
    return std::get<LoginResponse>(__data);
}

Response& Response::set_login(const LoginResponse& login) {
    __data = login;
    return *this;
}

LoginResponse& Response::make_login() {
    LoginResponse tmp;
    set_login(tmp);
    return std::get<LoginResponse>(__data);
}

const std::string_view Response::selection() const noexcept {
    const size_t index = __data.index();
    switch (index) {
        case 1:
            return "pong";
        case 2:
            return "login";
        default:
            return "unselected";
    }
}


const Request& Packet::get_request() const noexcept {
    return std::get<Request>(__data);
}

Packet& Packet::set_request(const Request& request) {
    __data = request;
    return *this;
}

Request& Packet::make_request() {
    Request tmp;
    set_request(tmp);
    return std::get<Request>(__data);
}

const Response& Packet::get_response() const noexcept {
    return std::get<Response>(__data);
}

Packet& Packet::set_response(const Response& response) {
    __data = response;
    return *this;
}

Response& Packet::make_response() {
    Response tmp;
    set_response(tmp);
    return std::get<Response>(__data);
}

const std::string_view Packet::selection() const noexcept {
    const size_t index = __data.index();
    switch (index) {
        case 1:
            return "request";
        case 2:
            return "response";
        default:
            return "unselected";
    }
}

} // namespace Packet
