#pragma once

// Some meaningful documentation

#include "packetfactory.h"
#include <string>

namespace RoseCommon {
namespace Packet {

REGISTER_RECV_PACKET(ePacketType::PAKCS_LOGIN_REQ, SrvLoginReq)
REGISTER_SEND_PACKET(ePacketType::PAKCS_LOGIN_REQ, SrvLoginReq)
class SrvLoginReq : public CRosePacket {
    public:
        SrvLoginReq();
        SrvLoginReq(CRoseReader reader);
        SrvLoginReq(SrvLoginReq&&) = default;
        SrvLoginReq& operator=(SrvLoginReq&&) = default;
        ~SrvLoginReq() = default;
        
        void set_password(Password&);
        Password& get_password() const;
        void set_username(std::string&);
        std::string& get_username() const;
        
        
        static SrvLoginReq create(Password, std::string,);
    
    private:
        Password password;
        std::string username;
};

}
}
