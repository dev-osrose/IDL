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
        
        
        struct Password : public ISerialize {
            explicit Password();
            explicit Password(std::string);
            Password(const Password&) = default;
            Password(Password&&) = default;
            virtual ~Password() = default;
            
            operator std::string() const { return password; }
            bool isValid() const { return is_valid; }
            
            virtual bool read(CRoseReader&) override;
            virtual bool write(CRoseBasePolicy&) const override;
            
            private:
                std::string password;
                bool is_valid;
        };
        
        
        void set_password(Password&);
        Password& get_password() const;
        void set_username(std::string&);
        std::string& get_username() const;
        
        
        static SrvLoginReq create(Password, std::string);
    
    protected:
        virtual void pack(CRoseBasePolicy&) const override;
    
    private:
        Password password;
        std::string username;
};

}
}
