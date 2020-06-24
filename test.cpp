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
    return std::get<static_cast<size_t>(Selection::SESSION_ID)>(__data);
}

LoginResponse& LoginResponse::set_sessionID(const std::string& sessionID) {
    __data.emplace<static_cast<size_t>(Selection::SESSION_ID)>(sessionID);
    return *this;
}

std::string& LoginResponse::make_sessionID() {
    std::string tmp;
    set_sessionID(tmp);
    return std::get<static_cast<size_t>(Selection::SESSION_ID)>(__data);
}

const LoginError& LoginResponse::get_error() const noexcept {
    return std::get<static_cast<size_t>(Selection::ERROR)>(__data);
}

LoginResponse& LoginResponse::set_error(const LoginError& error) {
    __data.emplace<static_cast<size_t>(Selection::ERROR)>(error);
    return *this;
}

LoginError& LoginResponse::make_error() {
    LoginError tmp;
    set_error(tmp);
    return std::get<static_cast<size_t>(Selection::ERROR)>(__data);
}

LoginResponse::Selection LoginResponse::selection() const noexcept {
    const size_t index = __data.index();
    return static_cast<Selection>(index);
}


const PingRequest& Request::get_ping() const noexcept {
    return std::get<static_cast<size_t>(Selection::PING)>(__data);
}

Request& Request::set_ping(const PingRequest& ping) {
    __data.emplace<static_cast<size_t>(Selection::PING)>(ping);
    return *this;
}

PingRequest& Request::make_ping() {
    PingRequest tmp;
    set_ping(tmp);
    return std::get<static_cast<size_t>(Selection::PING)>(__data);
}

const LoginRequest& Request::get_login() const noexcept {
    return std::get<static_cast<size_t>(Selection::LOGIN)>(__data);
}

Request& Request::set_login(const LoginRequest& login) {
    __data.emplace<static_cast<size_t>(Selection::LOGIN)>(login);
    return *this;
}

LoginRequest& Request::make_login() {
    LoginRequest tmp;
    set_login(tmp);
    return std::get<static_cast<size_t>(Selection::LOGIN)>(__data);
}

Request::Selection Request::selection() const noexcept {
    const size_t index = __data.index();
    return static_cast<Selection>(index);
}


const PongResponse& Response::get_pong() const noexcept {
    return std::get<static_cast<size_t>(Selection::PONG)>(__data);
}

Response& Response::set_pong(const PongResponse& pong) {
    __data.emplace<static_cast<size_t>(Selection::PONG)>(pong);
    return *this;
}

PongResponse& Response::make_pong() {
    PongResponse tmp;
    set_pong(tmp);
    return std::get<static_cast<size_t>(Selection::PONG)>(__data);
}

const LoginResponse& Response::get_login() const noexcept {
    return std::get<static_cast<size_t>(Selection::LOGIN)>(__data);
}

Response& Response::set_login(const LoginResponse& login) {
    __data.emplace<static_cast<size_t>(Selection::LOGIN)>(login);
    return *this;
}

LoginResponse& Response::make_login() {
    LoginResponse tmp;
    set_login(tmp);
    return std::get<static_cast<size_t>(Selection::LOGIN)>(__data);
}

Response::Selection Response::selection() const noexcept {
    const size_t index = __data.index();
    return static_cast<Selection>(index);
}


const Request& Packet::get_request() const noexcept {
    return std::get<static_cast<size_t>(Selection::REQUEST)>(__data);
}

Packet& Packet::set_request(const Request& request) {
    __data.emplace<static_cast<size_t>(Selection::REQUEST)>(request);
    return *this;
}

Request& Packet::make_request() {
    Request tmp;
    set_request(tmp);
    return std::get<static_cast<size_t>(Selection::REQUEST)>(__data);
}

const Response& Packet::get_response() const noexcept {
    return std::get<static_cast<size_t>(Selection::RESPONSE)>(__data);
}

Packet& Packet::set_response(const Response& response) {
    __data.emplace<static_cast<size_t>(Selection::RESPONSE)>(response);
    return *this;
}

Response& Packet::make_response() {
    Response tmp;
    set_response(tmp);
    return std::get<static_cast<size_t>(Selection::RESPONSE)>(__data);
}

Packet::Selection Packet::selection() const noexcept {
    const size_t index = __data.index();
    return static_cast<Selection>(index);
}

} // namespace Packet
