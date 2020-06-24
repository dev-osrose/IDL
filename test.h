/* Generated with IDL v0.2.0 */

#include <vector>
#include <string>
#include <array>
#include <variant>
#include <string_view>
#include <optional>

namespace Packet {

enum class LoginError : uint16_t {
    UNKNOWN_USER = 0,
    WRONG_PASSWORD = 1,
    SERVER_DOWN = 2,
};

struct PingRequest {};

struct PongResponse {};

class LoginRequest {
    public:
        const std::optional<std::string>& get_username() const noexcept;
        LoginRequest& set_username(const std::optional<std::string>& username);
        
        const std::array<char, 32>& get_password() const noexcept;
        LoginRequest& set_password(const std::array<char, 32>& password);
        
        
    private:
        std::optional<std::string> username;
        std::array<char, 32> password;
};

class LoginResponse {
    public:
        enum class Selection : size_t {
            UNSELECTED = 0,
            SESSION_ID = 1,
            ERROR = 2,
        };
        
        const std::string& get_sessionID() const noexcept;
        LoginResponse& set_sessionID(const std::string& sessionID);
        std::string& make_sessionID();
        
        const LoginError& get_error() const noexcept;
        LoginResponse& set_error(const LoginError& error);
        LoginError& make_error();
        
        Selection selection() const noexcept;
        
        const auto& visit() const noexcept { return __data; }
        
    private:
        std::variant<std::monostate, std::string, LoginError> __data;
};

class Request {
    public:
        enum class Selection : size_t {
            UNSELECTED = 0,
            PING = 1,
            LOGIN = 2,
        };
        
        const PingRequest& get_ping() const noexcept;
        Request& set_ping(const PingRequest& ping);
        PingRequest& make_ping();
        
        const LoginRequest& get_login() const noexcept;
        Request& set_login(const LoginRequest& login);
        LoginRequest& make_login();
        
        Selection selection() const noexcept;
        
        const auto& visit() const noexcept { return __data; }
        
    private:
        std::variant<std::monostate, PingRequest, LoginRequest> __data;
};

class Response {
    public:
        enum class Selection : size_t {
            UNSELECTED = 0,
            PONG = 1,
            LOGIN = 2,
        };
        
        const PongResponse& get_pong() const noexcept;
        Response& set_pong(const PongResponse& pong);
        PongResponse& make_pong();
        
        const LoginResponse& get_login() const noexcept;
        Response& set_login(const LoginResponse& login);
        LoginResponse& make_login();
        
        Selection selection() const noexcept;
        
        const auto& visit() const noexcept { return __data; }
        
    private:
        std::variant<std::monostate, PongResponse, LoginResponse> __data;
};

class Packet {
    public:
        enum class Selection : size_t {
            UNSELECTED = 0,
            REQUEST = 1,
            RESPONSE = 2,
        };
        
        const Request& get_request() const noexcept;
        Packet& set_request(const Request& request);
        Request& make_request();
        
        const Response& get_response() const noexcept;
        Packet& set_response(const Response& response);
        Response& make_response();
        
        Selection selection() const noexcept;
        
        const auto& visit() const noexcept { return __data; }
        
    private:
        std::variant<std::monostate, Request, Response> __data;
};
} // namespace Packet
