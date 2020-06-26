/* Generated with IDL v0.2.0 */

#include <vector>
#include <string>
#include <array>
#include <variant>
#include <string_view>
#include <optional>

namespace Packet {

enum class LoginError : uint16_t;
class PingRequest;
class PongResponse;
class LoginRequest;
class LoginResponse;
class Request;
class Response;

template <typename Derived>
struct VisitorBase {
    virtual ~VisitorBase() = default;
    virtual bool visit_sequence(size_t length) = 0;
    virtual bool visit_enum(uint16_t& data) = 0;
    bool visit_null() {
        std::monostate monostate;
        return (*this)(monostate);
    }
    virtual bool operator()(uint8_t&) = 0;
    virtual bool operator()(int8_t&) = 0;
    virtual bool operator()(uint16_t&) = 0;
    virtual bool operator()(int16_t&) = 0;
    virtual bool operator()(uint32_t&) = 0;
    virtual bool operator()(int32_t&) = 0;
    virtual bool operator()(uint64_t&) = 0;
    virtual bool operator()(int64_t&) = 0;
    virtual bool operator()(std::string&) = 0;
    virtual bool operator()(std::monostate&) = 0;
    template <typename T>
    bool operator(std::optional<T>& data) {
        return dynamic_cast<Derived&>(*this)(data);
    }
    template <typename T, size_t N>
    bool operator(std::array<T, N>& data) {
        return dynamic_cast<Derived&>(*this)(data);
    }
    template <typename T>
    bool operator()(std::vector<T>& data) {
        return dynamic_cast<Derived&>(*this)(data);
    }
    template <typename... Args>
    bool visit_choice(std::variant<Args...>& data) {
        return dynamic_cast<Derived&>(*this).visit_choice(data);
    }
    bool operator()(LoginError&);
    bool operator()(PingRequest&);
    bool operator()(PongResponse&);
    bool operator()(LoginRequest&);
    bool operator()(LoginResponse&);
    bool operator()(Request&);
    bool operator()(Response&);
};


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
        
        
        template <typename T>
        bool visit(VisitorBase<T>& v) {
            bool result = true;
            result &= v(username);
            result &= v(password);
            return result;
        }
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
        
        const auto& visit_inner() const noexcept { return __data; }
        
        template <typename T>
        bool visit(VisitorBase<T>& v) {
            bool result = true;
            result &= v(__data);
            return result;
        }
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
        
        const auto& visit_inner() const noexcept { return __data; }
        
        template <typename T>
        bool visit(VisitorBase<T>& v) {
            bool result = true;
            result &= v(__data);
            return result;
        }
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
        
        const auto& visit_inner() const noexcept { return __data; }
        
        template <typename T>
        bool visit(VisitorBase<T>& v) {
            bool result = true;
            result &= v(__data);
            return result;
        }
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
        
        const auto& visit_inner() const noexcept { return __data; }
        
        template <typename T>
        bool visit(VisitorBase<T>& v) {
            bool result = true;
            result &= v(__data);
            return result;
        }
    private:
        std::variant<std::monostate, Request, Response> __data;
};

template <typename T>
bool VisitorBase<T>::operator()(LoginError& data) {
    return this->visit_enum(static_cast<uint16_t&>(data)));
}
template <typename T>
bool VisitorBase<T>::operator()(PingRequest& data) {
    return visit_sequence(0);
}
template <typename T>
bool VisitorBase<T>::operator()(PongResponse& data) {
    return visit_sequence(0);
}
template <typename T>
bool VisitorBase<T>::operator()(LoginRequest& data) {
    bool result = visit_sequence(2);
    return result && data.visit(*this);
}
template <typename T>
bool VisitorBase<T>::operator()(LoginResponse& data) {
    return data.visit(*this);
}
template <typename T>
bool VisitorBase<T>::operator()(Request& data) {
    return data.visit(*this);
}
template <typename T>
bool VisitorBase<T>::operator()(Response& data) {
    return data.visit(*this);
}
} // namespace Packet
