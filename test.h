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
        
        struct Password {
            explicit Password(std::string);
            
            operator std::string() const;
            bool isValid() const;
            
            private:
                std::string password;
                bool is_valid;
        };
        
        
        static SrvLoginReq create(Password, std::string);
    
    protected:
        virtual void pack(CRosePolicyBase&) const override;
    
    private:
        Password password;
        std::string username;
};

}
}
