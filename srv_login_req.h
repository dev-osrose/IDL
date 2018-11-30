#pragma once


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
        
        
        
        void set_id(const uint32_t);
        const uint32_t get_id() const;
        void set_channels(const ChannelInfo&);
        const ChannelInfo& get_channels() const;
        
        
        static SrvLoginReq create(const uint32_t&);
    
    protected:
        virtual void pack(CRoseBasePolicy&) const override;
    
    private:
        uint32_t id;
        ChannelInfo channels;
};

}
}
